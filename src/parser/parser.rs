use crate::parser::ast::{Expression, InterpolationPart, Statement};
use crate::parser::error::ParseError;
use std::collections::VecDeque;
use std::panic;

pub(crate) fn parse_program(code: &str) -> Vec<Statement> {
    let mut lines: VecDeque<&str> = code
        .lines()
        .map(strip_inline_comment)
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect();
    parse_block(&mut lines, false)
}

pub(crate) fn try_parse_program(code: &str) -> Result<Vec<Statement>, ParseError> {
    let result = panic::catch_unwind(|| parse_program(code));
    match result {
        Ok(stmts) => Ok(stmts),
        Err(payload) => {
            let message = if let Some(s) = payload.downcast_ref::<&'static str>() {
                s.to_string()
            } else if let Some(s) = payload.downcast_ref::<String>() {
                s.clone()
            } else {
                "Parse error".to_string()
            };
            Err(ParseError { message })
        }
    }
}

enum BlockTerminator {
    End,
    Else(String),
    Catch(String),
}

fn parse_block(lines: &mut VecDeque<&str>, expect_end: bool) -> Vec<Statement> {
    parse_block_with_terminators(lines, expect_end, false, false).0
}

fn parse_block_with_terminators(
    lines: &mut VecDeque<&str>,
    expect_end: bool,
    allow_else: bool,
    allow_catch: bool,
) -> (Vec<Statement>, Option<BlockTerminator>) {
    let mut statements = Vec::new();

    while let Some(line) = lines.pop_front() {
        match line.split_whitespace().next() {
            Some("set") => statements.push(parse_set_statement(line, lines)),
            Some("function") | Some("export") => statements.push(parse_function(lines, line)),
            Some("if") => statements.push(parse_if_statement(lines, line)),
            Some("try") => statements.push(parse_try_catch_statement(lines, line)),
            Some("print") => statements.push(parse_print_statement(line)),
            Some("return") => statements.push(parse_return_statement(line)),
            Some("break") => statements.push(parse_break_statement(line)),
            Some("continue") => statements.push(parse_continue_statement(line)),
            Some("for") => statements.push(parse_for_loop(lines, line)),
            Some("while") => statements.push(parse_while_loop(lines, line)),
            Some("import") => statements.push(parse_import_statement(line)),
            Some("test") => statements.push(parse_test_block(lines, line)),
            Some("else") => {
                if allow_else {
                    return (statements, Some(BlockTerminator::Else(line.to_string())));
                }
                panic!("Unexpected else");
            }
            Some("catch") => {
                if allow_catch {
                    return (statements, Some(BlockTerminator::Catch(line.to_string())));
                }
                panic!("Unexpected catch");
            }
            Some("end") => {
                if expect_end {
                    return (statements, Some(BlockTerminator::End));
                }
                panic!("Unexpected end");
            }
            _ => {
                if line.contains('(') && line.contains(')') {
                    statements.push(parse_function_call_statement(line));
                }
            }
        }
    }

    if expect_end {
        panic!("Missing end for block");
    }

    (statements, None)
}

fn strip_required_start_suffix<'a>(header: &'a str, statement: &str) -> &'a str {
    let trimmed = header.trim_end();
    trimmed
        .strip_suffix(" start")
        .unwrap_or_else(|| panic!("{} statement must end with 'start': {}", statement, header))
        .trim_end()
}

fn parse_function_call_statement(expr: &str) -> Statement {
    let name = extract_before(expr, "(").trim().to_string();
    let args_str = extract_between(expr, "(", ")");
    let args = split_top_level(args_str, ',')
        .into_iter()
        .filter(|arg| !arg.trim().is_empty())
        .map(|arg| parse_expression(arg.trim()))
        .collect();

    Statement::FunctionCall { name, args }
}

fn parse_set_statement(first_line: &str, lines: &mut VecDeque<&str>) -> Statement {
    let parts: Vec<&str> = first_line.splitn(3, ' ').collect();
    if parts.len() < 3 {
        panic!("Invalid set statement: {}", first_line);
    }

    let target = parts[1].to_string();
    let mut value_str = parts[2].to_string();

    if value_str == "{" {
        let mut lines_to_parse = Vec::new();
        let mut brace_depth = 1;

        while let Some(line) = lines.pop_front() {
            let trimmed = line.trim();

            let mut in_string = false;
            let mut escaped = false;
            for ch in trimmed.chars() {
                if escaped {
                    escaped = false;
                    continue;
                }
                match ch {
                    '\\' => escaped = true,
                    '"' => in_string = !in_string,
                    '{' if !in_string => brace_depth += 1,
                    '}' if !in_string => brace_depth -= 1,
                    _ => {}
                }
            }

            if trimmed == "}" && brace_depth == 0 {
                break;
            } else if brace_depth == 0 {
                lines_to_parse.push(trimmed);
                break;
            } else {
                lines_to_parse.push(trimmed);
            }
        }

        let mut result = String::from("{");
        for (i, line) in lines_to_parse.iter().enumerate() {
            if i > 0 && !result.ends_with('{') && !result.ends_with(' ') {
                result.push(' ');
            }
            result.push_str(line);
        }
        result.push('}');

        value_str = result;
    }

    let value = parse_expression(&value_str);

    if target.contains('.') {
        let dot_pos = target.rfind('.').unwrap();
        let object_path = &target[..dot_pos];
        let property = target[dot_pos + 1..].to_string();

        let object_expr = if object_path.contains('.') {
            parse_property_access(object_path)
        } else {
            Expression::Variable(object_path.to_string())
        };

        return Statement::PropertySet {
            object: object_expr,
            property,
            value,
        };
    }

    Statement::Set { var: target, value }
}

fn parse_function(lines: &mut VecDeque<&str>, header: &str) -> Statement {
    let exported = header.starts_with("export");
    let header = header.trim_start_matches("export ");
    let header = strip_required_start_suffix(header, "function");
    let name = extract_between(header, "function", "(").trim().to_string();
    let params = extract_between(header, "(", ")")
        .split(',')
        .map(str::trim)
        .filter(|param| !param.is_empty())
        .map(String::from)
        .collect();

    let body = parse_block(lines, true);

    Statement::Function {
        name,
        params,
        body,
        exported,
    }
}

fn parse_if_statement(lines: &mut VecDeque<&str>, header: &str) -> Statement {
    let header = strip_required_start_suffix(header, "if");
    let condition = parse_expression(header[3..].trim());
    let (body, terminator) = parse_block_with_terminators(lines, true, true, false);
    let else_body = match terminator {
        Some(BlockTerminator::End) => None,
        Some(BlockTerminator::Else(line)) => Some(parse_else_branch(lines, &line)),
        Some(BlockTerminator::Catch(_)) | None => panic!("Missing end for if block"),
    };

    Statement::If {
        condition,
        body,
        else_body,
    }
}

fn parse_print_statement(line: &str) -> Statement {
    Statement::Print {
        expr: parse_expression(line[6..].trim()),
    }
}

fn parse_return_statement(line: &str) -> Statement {
    let expr_str = line.trim();
    if expr_str.len() > 6 {
        Statement::Return {
            expr: parse_expression(expr_str[6..].trim()),
        }
    } else {
        Statement::Return {
            expr: Expression::Null,
        }
    }
}

fn parse_break_statement(line: &str) -> Statement {
    if line.trim() != "break" {
        panic!("Invalid break statement: {}", line);
    }
    Statement::Break
}

fn parse_continue_statement(line: &str) -> Statement {
    if line.trim() != "continue" {
        panic!("Invalid continue statement: {}", line);
    }
    Statement::Continue
}

fn parse_import_statement(line: &str) -> Statement {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 4 || parts[2] != "from" {
        panic!("Invalid import syntax. Expected: import module_name from 'file_path'");
    }

    let module_name = parts[1].to_string();
    let file_path_with_quotes = parts[3..].join(" ");

    let file_path = if (file_path_with_quotes.starts_with('\'')
        && file_path_with_quotes.ends_with('\''))
        || (file_path_with_quotes.starts_with('"') && file_path_with_quotes.ends_with('"'))
    {
        file_path_with_quotes[1..file_path_with_quotes.len() - 1].to_string()
    } else {
        panic!("File path must be quoted in import statement");
    };

    Statement::Import {
        module_name,
        file_path,
    }
}

fn parse_else_branch(lines: &mut VecDeque<&str>, line: &str) -> Vec<Statement> {
    if line.trim() == "else start" {
        return parse_block(lines, true);
    }

    let else_if_header = line
        .trim()
        .strip_prefix("else ")
        .unwrap_or_else(|| panic!("Invalid else statement: {}", line));

    if !else_if_header.starts_with("if ") {
        panic!("Invalid else statement: {}", line);
    }

    vec![parse_if_statement(lines, else_if_header)]
}

fn parse_try_catch_statement(lines: &mut VecDeque<&str>, header: &str) -> Statement {
    let try_header = strip_required_start_suffix(header, "try");
    if try_header.trim() != "try" {
        panic!("Invalid try statement: {}", header);
    }

    let (try_body, terminator) = parse_block_with_terminators(lines, true, false, true);
    let catch_header = match terminator {
        Some(BlockTerminator::Catch(line)) => line,
        Some(BlockTerminator::End) | Some(BlockTerminator::Else(_)) | None => {
            panic!("try block must be followed by catch")
        }
    };

    let catch_header = strip_required_start_suffix(&catch_header, "catch");
    let error_var = catch_header
        .strip_prefix("catch")
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("error")
        .to_string();
    let catch_body = parse_block(lines, true);

    Statement::TryCatch {
        try_body,
        error_var,
        catch_body,
    }
}

fn parse_expression(expr: &str) -> Expression {
    let expr = expr.trim();

    if let Ok(num) = expr.parse::<i32>() {
        return Expression::Number(num);
    }

    if expr == "true" || expr == "false" {
        return Expression::Boolean(expr == "true");
    }

    if expr == "null" {
        return Expression::Null;
    }

    if expr.starts_with('"') && expr.ends_with('"') {
        let content = &expr[1..expr.len() - 1];
        if content.contains("${") {
            return parse_interpolated_string(content);
        } else {
            return Expression::StringLiteral(content.to_string());
        }
    }

    if expr.starts_with('[') && expr.ends_with(']') {
        return parse_array(expr);
    }

    if expr.starts_with('{') && expr.ends_with('}') {
        return parse_object(expr);
    }

    if expr.contains('+') {
        let parts: Vec<&str> = expr.split('+').map(str::trim).collect();
        if parts.len() >= 2 {
            let mut current = parse_expression(parts[0]);
            for part in &parts[1..] {
                let right = parse_expression(part);
                current = Expression::FunctionCall {
                    name: "string_concat".to_string(),
                    args: vec![current, right],
                };
            }
            return current;
        }
    }

    if let Some((operator_pos, operator)) = find_top_level_comparison_operator(expr) {
        let left = parse_expression(expr[..operator_pos].trim());
        let right = parse_expression(expr[operator_pos + operator.len()..].trim());
        return Expression::Comparison {
            left: Box::new(left),
            operator: operator.to_string(),
            right: Box::new(right),
        };
    }

    if expr.contains('(') && expr.contains(')') {
        return parse_function_call(expr);
    }

    if expr.contains('.') {
        return parse_property_access(expr);
    }

    Expression::Variable(expr.to_string())
}

fn parse_object(expr: &str) -> Expression {
    let content = extract_between(expr, "{", "}");
    let mut properties = std::collections::HashMap::new();

    let parts = split_top_level(content, ',');

    for part in parts.into_iter().filter_map(|p| {
        let trimmed = p.trim().to_string();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    }) {
        let key_value: Vec<&str> = split_top_level_once(&part, ':');

        if key_value.len() != 2 {
            panic!("Invalid object syntax: {}", expr);
        }
        let key = key_value[0].trim().to_string();
        let value_str = key_value[1].trim();
        let value = parse_expression(value_str);
        properties.insert(key, value);
    }

    Expression::Object(properties)
}

fn parse_property_access(expr: &str) -> Expression {
    let parts: Vec<&str> = expr.split('.').collect();
    if parts.len() < 2 {
        panic!("Invalid property access: {}", expr);
    }

    let mut object_expr = Expression::Variable(parts[0].to_string());

    for property in &parts[1..] {
        object_expr = Expression::PropertyAccess {
            object: Box::new(object_expr),
            property: property.to_string(),
        };
    }

    object_expr
}

fn parse_for_loop(lines: &mut VecDeque<&str>, header: &str) -> Statement {
    let header = strip_required_start_suffix(header, "for");
    let parts: Vec<&str> = header.split_whitespace().collect();
    if parts[2] != "in" {
        panic!("Invalid for loop syntax: {}", header);
    }

    let variable = parts[1].to_string();
    let bind = parts[3..].join(" ");
    let iterable = parse_expression(&bind);
    let body = parse_block(lines, true);

    Statement::ForLoop {
        variable,
        iterable,
        body,
    }
}

fn parse_while_loop(lines: &mut VecDeque<&str>, header: &str) -> Statement {
    let header = strip_required_start_suffix(header, "while");
    let condition_str = header[6..].trim();
    let condition = parse_expression(condition_str);
    let body = parse_block(lines, true);

    Statement::WhileLoop { condition, body }
}

fn parse_array(expr: &str) -> Expression {
    let content = extract_between(expr, "[", "]");
    let elements: Vec<Expression> = split_top_level(content, ',')
        .into_iter()
        .filter_map(|p| {
            let trimmed = p.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        })
        .map(|s| parse_expression(&s))
        .collect();

    Expression::Array(elements)
}

fn parse_function_call(expr: &str) -> Expression {
    let name = extract_before(expr, "(").trim().to_string();
    let args_str = extract_between(expr, "(", ")");
    let args = split_top_level(args_str, ',')
        .into_iter()
        .filter(|arg| !arg.trim().is_empty())
        .map(|arg| parse_expression(arg.trim()))
        .collect();

    Expression::FunctionCall { name, args }
}

fn parse_test_block(lines: &mut VecDeque<&str>, header: &str) -> Statement {
    let header = strip_required_start_suffix(header, "test");
    let name_start = header.find('"').unwrap_or(0);
    let name_end = header[name_start + 1..]
        .find('"')
        .map(|idx| idx + name_start + 1)
        .unwrap_or(name_start);
    let name = if name_start > 0 && name_end > name_start {
        header[name_start + 1..name_end].to_string()
    } else {
        "Unnamed Test".to_string()
    };

    let body = parse_block(lines, true);

    Statement::Test { name, body }
}

fn extract_between<'a>(s: &'a str, start: &str, end: &str) -> &'a str {
    let start_pos = match s.find(start) {
        Some(pos) => pos + start.len(),
        None => return "",
    };

    if (start == "{" && end == "}") || (start == "[" && end == "]") || (start == "(" && end == ")")
    {
        let start_ch = start.chars().next().unwrap_or_default();
        let end_ch = end.chars().next().unwrap_or_default();
        let mut depth = 1;
        let mut in_string = false;
        let mut escaped = false;
        let content = &s[start_pos..];

        for (i, ch) in content.char_indices() {
            if escaped {
                escaped = false;
                continue;
            }

            match ch {
                '\\' => escaped = true,
                '"' => in_string = !in_string,
                c if c == start_ch && !in_string => depth += 1,
                c if c == end_ch && !in_string => {
                    depth -= 1;
                    if depth == 0 {
                        return &content[..i];
                    }
                }
                _ => {}
            }
        }
        return content;
    }

    s[start_pos..].split(end).next().unwrap_or(&s[start_pos..])
}

fn extract_before<'a>(s: &'a str, delimiter: &str) -> &'a str {
    s.split(delimiter).next().unwrap_or(s)
}

fn split_top_level(s: &str, delimiter: char) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut depth_curly = 0;
    let mut depth_square = 0;
    let mut depth_paren = 0;
    let mut in_string = false;
    for c in s.chars() {
        match c {
            '"' => {
                current.push(c);
                let prev_backslash = current.chars().rev().nth(1) == Some('\\');
                if !prev_backslash {
                    in_string = !in_string;
                }
            }
            '{' if !in_string => {
                depth_curly += 1;
                current.push(c);
            }
            '}' if !in_string => {
                depth_curly -= 1;
                current.push(c);
            }
            '[' if !in_string => {
                depth_square += 1;
                current.push(c);
            }
            ']' if !in_string => {
                depth_square -= 1;
                current.push(c);
            }
            '(' if !in_string => {
                depth_paren += 1;
                current.push(c);
            }
            ')' if !in_string => {
                depth_paren -= 1;
                current.push(c);
            }
            d if d == delimiter
                && !in_string
                && depth_curly == 0
                && depth_square == 0
                && depth_paren == 0 =>
            {
                parts.push(current.trim().to_string());
                current.clear();
            }
            _ => current.push(c),
        }
    }
    if !current.trim().is_empty() {
        parts.push(current.trim().to_string());
    }
    parts
}

fn split_top_level_once(s: &str, delimiter: char) -> Vec<&str> {
    let mut depth_curly = 0;
    let mut depth_square = 0;
    let mut depth_paren = 0;
    let mut in_string = false;
    for (i, c) in s.char_indices() {
        match c {
            '"' => {
                let mut backslashes = 0;
                let mut idx = i;
                while idx > 0 && s.as_bytes()[idx - 1] == b'\\' {
                    backslashes += 1;
                    idx -= 1;
                }
                if backslashes % 2 == 0 {
                    in_string = !in_string;
                }
            }
            '{' if !in_string => depth_curly += 1,
            '}' if !in_string => depth_curly -= 1,
            '[' if !in_string => depth_square += 1,
            ']' if !in_string => depth_square -= 1,
            '(' if !in_string => depth_paren += 1,
            ')' if !in_string => depth_paren -= 1,
            d if d == delimiter
                && !in_string
                && depth_curly == 0
                && depth_square == 0
                && depth_paren == 0 =>
            {
                return vec![s[..i].trim(), s[i + 1..].trim()];
            }
            _ => {}
        }
    }
    vec![s.trim()]
}

fn strip_inline_comment(line: &str) -> &str {
    let mut in_string = false;
    let mut escaped = false;

    for (idx, ch) in line.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }

        match ch {
            '\\' if in_string => escaped = true,
            '"' => in_string = !in_string,
            '#' if !in_string => return &line[..idx],
            _ => {}
        }
    }

    line
}

fn find_top_level_comparison_operator(expr: &str) -> Option<(usize, &'static str)> {
    let operators = ["==", "!=", "<=", ">=", "<", ">"];
    let mut depth_curly = 0;
    let mut depth_square = 0;
    let mut depth_paren = 0;
    let mut in_string = false;
    let mut escaped = false;

    for (idx, ch) in expr.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }

        match ch {
            '\\' if in_string => escaped = true,
            '"' => in_string = !in_string,
            '{' if !in_string => depth_curly += 1,
            '}' if !in_string => depth_curly -= 1,
            '[' if !in_string => depth_square += 1,
            ']' if !in_string => depth_square -= 1,
            '(' if !in_string => depth_paren += 1,
            ')' if !in_string => depth_paren -= 1,
            _ => {
                if !in_string && depth_curly == 0 && depth_square == 0 && depth_paren == 0 {
                    for op in operators {
                        if expr[idx..].starts_with(op) {
                            return Some((idx, op));
                        }
                    }
                }
            }
        }
    }

    None
}

fn parse_interpolated_string(content: &str) -> Expression {
    let mut parts = Vec::new();
    let mut current_text = String::new();
    let mut chars = content.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '$' && chars.peek() == Some(&'{') {
            if !current_text.is_empty() {
                parts.push(InterpolationPart::Text(current_text.clone()));
                current_text.clear();
            }

            chars.next();
            let expr_str = extract_balanced_braces(&mut chars);
            let expr = parse_expression(&expr_str);
            parts.push(InterpolationPart::Expression(expr));
        } else if ch == '\\' && chars.peek() == Some(&'$') {
            chars.next();
            current_text.push('$');
        } else {
            current_text.push(ch);
        }
    }

    if !current_text.is_empty() {
        parts.push(InterpolationPart::Text(current_text));
    }

    Expression::StringInterpolation { parts }
}

fn extract_balanced_braces(chars: &mut std::iter::Peekable<std::str::Chars>) -> String {
    let mut result = String::new();
    let mut depth = 1;
    let mut in_string = false;
    let mut escape_next = false;

    for ch in chars.by_ref() {
        if escape_next {
            result.push(ch);
            escape_next = false;
            continue;
        }

        match ch {
            '\\' if in_string => {
                result.push(ch);
                escape_next = true;
            }
            '"' => {
                result.push(ch);
                in_string = !in_string;
            }
            '{' if !in_string => {
                result.push(ch);
                depth += 1;
            }
            '}' if !in_string => {
                depth -= 1;
                if depth == 0 {
                    break;
                }
                result.push(ch);
            }
            _ => {
                result.push(ch);
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::{parse_program, try_parse_program};
    use crate::parser::ast::{Expression, Statement};

    #[test]
    fn parse_program_keeps_hash_inside_strings() {
        let statements = parse_program("set s \"hello # world\"\nprint s\n");
        assert_eq!(statements.len(), 2);

        match &statements[0] {
            Statement::Set {
                var,
                value: Expression::StringLiteral(value),
            } => {
                assert_eq!(var, "s");
                assert_eq!(value, "hello # world");
            }
            _ => panic!("Expected set statement with string literal"),
        }
    }

    #[test]
    fn parse_expression_handles_nested_comparison_in_function_arg() {
        let statements = parse_program("set x type_of(1 > 0)\n");
        assert_eq!(statements.len(), 1);

        match &statements[0] {
            Statement::Set { var, value } => {
                assert_eq!(var, "x");
                match value {
                    Expression::FunctionCall { name, args } => {
                        assert_eq!(name, "type_of");
                        assert_eq!(args.len(), 1);
                        assert!(matches!(args[0], Expression::Comparison { .. }));
                    }
                    _ => panic!("Expected function call expression"),
                }
            }
            _ => panic!("Expected set statement"),
        }
    }

    #[test]
    fn parse_program_rejects_if_without_start() {
        let result = try_parse_program("if true\nprint 1\nend\n");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("if statement must end with 'start'"));
    }

    #[test]
    fn parse_program_rejects_missing_end_for_block() {
        let result = try_parse_program("function add(x) start\nreturn x\n");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("Missing end for block"));
    }

    #[test]
    fn parse_program_rejects_unexpected_end() {
        let result = try_parse_program("end\n");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("Unexpected end"));
    }

    #[test]
    fn parse_program_handles_utf8_object_keys_without_panicking() {
        let result = try_parse_program("set obj {føø: 1}\n");
        assert!(result.is_ok());
    }

    #[test]
    fn parse_program_handles_utf8_inside_balanced_delimiters_without_panicking() {
        let result = try_parse_program("set arr [\"é\"]\n");
        assert!(result.is_ok());
    }

    #[test]
    fn parse_program_recognizes_null_literal() {
        let statements = parse_program("set value null\n");
        assert!(matches!(
            &statements[0],
            Statement::Set {
                var,
                value: Expression::Null
            } if var == "value"
        ));
    }

    #[test]
    fn parse_program_handles_functions_without_parameters() {
        let statements = parse_program("function outer() start\nprint \"ok\"\nend\n");
        assert!(matches!(
            &statements[0],
            Statement::Function { name, params, .. } if name == "outer" && params.is_empty()
        ));
    }

    #[test]
    fn parse_program_handles_else_if_and_else() {
        let statements = parse_program(
            "if true start\nprint 1\nelse if false start\nprint 2\nelse start\nprint 3\nend\n",
        );
        assert!(matches!(
            &statements[0],
            Statement::If {
                else_body: Some(_),
                ..
            }
        ));
    }

    #[test]
    fn parse_program_handles_try_catch() {
        let statements =
            parse_program("try start\nprint missing\ncatch err start\nprint err\nend\n");
        assert!(matches!(
            &statements[0],
            Statement::TryCatch {
                error_var,
                catch_body,
                ..
            } if error_var == "err" && !catch_body.is_empty()
        ));
    }
}
