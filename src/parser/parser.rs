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
    parse_block(&mut lines)
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

fn parse_block(lines: &mut VecDeque<&str>) -> Vec<Statement> {
    let mut statements = Vec::new();

    while let Some(line) = lines.pop_front() {
        match line.split_whitespace().next() {
            Some("set") => statements.push(parse_set_statement(line, lines)),
            Some("function") | Some("export") => statements.push(parse_function(lines, line)),
            Some("if") => statements.push(parse_if_statement(lines, line)),
            Some("print") => statements.push(parse_print_statement(line)),
            Some("return") => statements.push(parse_return_statement(line)),
            Some("for") => statements.push(parse_for_loop(lines, line)),
            Some("while") => statements.push(parse_while_loop(lines, line)),
            Some("import") => statements.push(parse_import_statement(line)),
            Some("test") => statements.push(parse_test_block(lines, line)),
            Some("end") => break,
            _ => {
                if line.contains('(') && line.contains(')') {
                    statements.push(parse_function_call_statement(line));
                }
            }
        }
    }

    statements
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
            if i > 0 {
                if !result.ends_with("{") && !result.ends_with(" ") {
                    result.push_str(" ");
                }
            }
            result.push_str(line);
        }
        result.push_str("}");

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
    let name = extract_between(header, "function", "(").trim().to_string();
    let params = extract_between(header, "(", ")")
        .split(',')
        .map(str::trim)
        .map(String::from)
        .collect();

    let body = parse_block(lines);

    Statement::Function {
        name,
        params,
        body,
        exported,
    }
}

fn parse_if_statement(lines: &mut VecDeque<&str>, header: &str) -> Statement {
    let condition = parse_expression(&header[3..].trim());
    let body = parse_block(lines);

    Statement::If { condition, body }
}

fn parse_print_statement(line: &str) -> Statement {
    Statement::Print {
        expr: parse_expression(&line[6..].trim()),
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

fn parse_expression(expr: &str) -> Expression {
    let expr = expr.trim();

    if let Ok(num) = expr.parse::<i32>() {
        return Expression::Number(num);
    }

    if expr == "true" || expr == "false" {
        return Expression::Boolean(expr == "true");
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
    let parts: Vec<&str> = header.split_whitespace().collect();
    if parts[2] != "in" {
        panic!("Invalid for loop syntax: {}", header);
    }

    let variable = parts[1].to_string();
    let mut bind = parts[3..].join(" ");
    bind = bind.strip_suffix(" start").unwrap_or(&bind).to_string();
    let iterable = parse_expression(&bind);
    let body = parse_block(lines);

    Statement::ForLoop {
        variable,
        iterable,
        body,
    }
}

fn parse_while_loop(lines: &mut VecDeque<&str>, header: &str) -> Statement {
    let condition_str = header[6..].trim();
    let condition_str = condition_str
        .strip_suffix(" start")
        .unwrap_or(condition_str);
    let condition = parse_expression(condition_str);
    let body = parse_block(lines);

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

    let body = parse_block(lines);

    Statement::Test { name, body }
}

fn extract_between<'a>(s: &'a str, start: &str, end: &str) -> &'a str {
    let start_pos = match s.find(start) {
        Some(pos) => pos + start.len(),
        None => return "",
    };

    if (start == "{" && end == "}") || (start == "[" && end == "]") || (start == "(" && end == ")")
    {
        let mut depth = 1;
        let mut in_string = false;
        let mut escaped = false;
        let chars: Vec<char> = s[start_pos..].chars().collect();

        for (i, &ch) in chars.iter().enumerate() {
            if escaped {
                escaped = false;
                continue;
            }

            match ch {
                '\\' => escaped = true,
                '"' => in_string = !in_string,
                c if c.to_string() == start && !in_string => depth += 1,
                c if c.to_string() == end && !in_string => {
                    depth -= 1;
                    if depth == 0 {
                        return &s[start_pos..start_pos + i];
                    }
                }
                _ => {}
            }
        }
        return &s[start_pos..];
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
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
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
    for (i, c) in s.chars().enumerate() {
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

    while let Some(ch) = chars.next() {
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
    use super::parse_program;
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
}
