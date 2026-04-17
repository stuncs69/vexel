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
    let Some((target, value_part)) = split_set_target_and_value(first_line) else {
        panic!("Invalid set statement: {}", first_line);
    };
    let mut value_str = value_part.to_string();

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

    match parse_expression(target) {
        Expression::Variable(var) => Statement::Set { var, value },
        Expression::PropertyAccess { object, property } => Statement::PropertySet {
            object: *object,
            property: *property,
            value,
        },
        _ => panic!("Invalid set target: {}", target),
    }
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
    parse_comparison(expr.trim())
}

fn parse_comparison(expr: &str) -> Expression {
    if let Some((operator_pos, operator)) =
        find_top_level_binary_operator(expr, &["==", "!=", "<=", ">=", "<", ">"])
    {
        let left = parse_bitwise_or(expr[..operator_pos].trim());
        let right = parse_bitwise_or(expr[operator_pos + operator.len()..].trim());
        return Expression::Comparison {
            left: Box::new(left),
            operator: operator.to_string(),
            right: Box::new(right),
        };
    }

    parse_bitwise_or(expr)
}

fn parse_bitwise_or(expr: &str) -> Expression {
    if let Some((operator_pos, operator)) = find_top_level_binary_operator(expr, &["|"]) {
        return Expression::BinaryOperation {
            left: Box::new(parse_bitwise_or(expr[..operator_pos].trim())),
            operator: operator.to_string(),
            right: Box::new(parse_bitwise_and(
                expr[operator_pos + operator.len()..].trim(),
            )),
        };
    }

    parse_bitwise_and(expr)
}

fn parse_bitwise_and(expr: &str) -> Expression {
    if let Some((operator_pos, operator)) = find_top_level_binary_operator(expr, &["&"]) {
        return Expression::BinaryOperation {
            left: Box::new(parse_bitwise_and(expr[..operator_pos].trim())),
            operator: operator.to_string(),
            right: Box::new(parse_shift(expr[operator_pos + operator.len()..].trim())),
        };
    }

    parse_shift(expr)
}

fn parse_shift(expr: &str) -> Expression {
    if let Some((operator_pos, operator)) = find_top_level_binary_operator(expr, &["<<", ">>"]) {
        return Expression::BinaryOperation {
            left: Box::new(parse_shift(expr[..operator_pos].trim())),
            operator: operator.to_string(),
            right: Box::new(parse_additive(expr[operator_pos + operator.len()..].trim())),
        };
    }

    parse_additive(expr)
}

fn parse_additive(expr: &str) -> Expression {
    if let Some((operator_pos, operator)) = find_top_level_binary_operator(expr, &["+", "-"]) {
        return Expression::BinaryOperation {
            left: Box::new(parse_additive(expr[..operator_pos].trim())),
            operator: operator.to_string(),
            right: Box::new(parse_multiplicative(
                expr[operator_pos + operator.len()..].trim(),
            )),
        };
    }

    parse_multiplicative(expr)
}

fn parse_multiplicative(expr: &str) -> Expression {
    if let Some((operator_pos, operator)) = find_top_level_binary_operator(expr, &["*", "/", "%"]) {
        return Expression::BinaryOperation {
            left: Box::new(parse_multiplicative(expr[..operator_pos].trim())),
            operator: operator.to_string(),
            right: Box::new(parse_unary(expr[operator_pos + operator.len()..].trim())),
        };
    }

    parse_unary(expr)
}

fn parse_unary(expr: &str) -> Expression {
    let expr = expr.trim();

    if let Some(rest) = expr.strip_prefix('~') {
        return Expression::UnaryOperation {
            operator: "~".to_string(),
            expr: Box::new(parse_unary(rest.trim())),
        };
    }

    if should_parse_as_unary_minus(expr) {
        return Expression::UnaryOperation {
            operator: "-".to_string(),
            expr: Box::new(parse_unary(expr[1..].trim())),
        };
    }

    parse_primary(expr)
}

fn parse_primary(expr: &str) -> Expression {
    let trimmed = expr.trim();
    let expr = strip_grouping_parentheses(trimmed);
    if expr != trimmed {
        return parse_expression(expr);
    }

    parse_postfix_expression(expr)
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

fn parse_postfix_expression(expr: &str) -> Expression {
    let (mut current, mut index) = parse_base_expression(expr);

    while index < expr.len() {
        index = skip_whitespace(expr, index);
        if index >= expr.len() {
            break;
        }

        let suffix = &expr[index..];
        if suffix.starts_with('.') {
            index += 1;
            index = skip_whitespace(expr, index);
            let property_start = index;
            index += consume_identifier(&expr[index..]);
            if property_start == index {
                panic!("Invalid property access: {}", expr);
            }

            current = Expression::PropertyAccess {
                object: Box::new(current),
                property: Box::new(Expression::StringLiteral(
                    expr[property_start..index].to_string(),
                )),
            };
            continue;
        }

        if suffix.starts_with('[') {
            let end = find_matching_delimiter_end(expr, index, '[', ']')
                .unwrap_or_else(|| panic!("Invalid bracket access: {}", expr));
            let property_expr = parse_expression(&expr[index + 1..end - 1]);
            current = Expression::PropertyAccess {
                object: Box::new(current),
                property: Box::new(property_expr),
            };
            index = end;
            continue;
        }

        panic!("Invalid expression: {}", expr);
    }

    current
}

fn parse_base_expression(expr: &str) -> (Expression, usize) {
    let expr = expr.trim();
    if expr.is_empty() {
        panic!("Invalid expression: {}", expr);
    }

    let first = expr.chars().next().unwrap();
    match first {
        '"' => {
            let end = find_string_end(expr).unwrap_or_else(|| panic!("Unterminated string: {}", expr));
            let content = &expr[1..end - 1];
            let expression = if content.contains("${") {
                parse_interpolated_string(content)
            } else {
                Expression::StringLiteral(content.to_string())
            };
            (expression, end)
        }
        '[' => {
            let end = find_matching_delimiter_end(expr, 0, '[', ']')
                .unwrap_or_else(|| panic!("Invalid array syntax: {}", expr));
            (parse_array(&expr[..end]), end)
        }
        '{' => {
            let end = find_matching_delimiter_end(expr, 0, '{', '}')
                .unwrap_or_else(|| panic!("Invalid object syntax: {}", expr));
            (parse_object(&expr[..end]), end)
        }
        '(' => {
            let end = find_matching_delimiter_end(expr, 0, '(', ')')
                .unwrap_or_else(|| panic!("Invalid grouping expression: {}", expr));
            (parse_expression(&expr[1..end - 1]), end)
        }
        _ if first.is_ascii_digit() => {
            let length = consume_digits(expr);
            let number = expr[..length]
                .parse::<i32>()
                .unwrap_or_else(|_| panic!("Invalid number literal: {}", expr));
            (Expression::Number(number), length)
        }
        _ => {
            let first_identifier_len = consume_identifier(expr);
            if first_identifier_len == 0 {
                panic!("Invalid expression: {}", expr);
            }

            let dotted_head_len = consume_dotted_identifier_chain(expr);
            let dotted_head_end = skip_whitespace(expr, dotted_head_len);
            if dotted_head_end < expr.len() && expr[dotted_head_end..].starts_with('(') {
                let call_end = find_matching_delimiter_end(expr, dotted_head_end, '(', ')')
                    .unwrap_or_else(|| panic!("Invalid function call: {}", expr));
                return (parse_function_call(&expr[..call_end]), call_end);
            }

            let identifier = &expr[..first_identifier_len];
            let expression = match identifier {
                "true" => Expression::Boolean(true),
                "false" => Expression::Boolean(false),
                "null" => Expression::Null,
                _ => Expression::Variable(identifier.to_string()),
            };
            (expression, first_identifier_len)
        }
    }
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

fn split_set_target_and_value(line: &str) -> Option<(&str, &str)> {
    let remainder = line.strip_prefix("set ")?;
    let mut depth_curly = 0;
    let mut depth_square = 0;
    let mut depth_paren = 0;
    let mut in_string = false;
    let mut escaped = false;

    for (idx, ch) in remainder.char_indices() {
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
            c if c.is_whitespace()
                && !in_string
                && depth_curly == 0
                && depth_square == 0
                && depth_paren == 0 =>
            {
                let target = remainder[..idx].trim();
                let value = remainder[idx..].trim();
                if target.is_empty() || value.is_empty() {
                    return None;
                }
                return Some((target, value));
            }
            _ => {}
        }
    }

    None
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

fn strip_grouping_parentheses(mut expr: &str) -> &str {
    loop {
        let trimmed = expr.trim();
        if trimmed.len() < 2 || !trimmed.starts_with('(') || !trimmed.ends_with(')') {
            return trimmed;
        }

        let mut depth = 0;
        let mut in_string = false;
        let mut escaped = false;
        let mut wraps_entire_expression = true;

        for (idx, ch) in trimmed.char_indices() {
            if escaped {
                escaped = false;
                continue;
            }

            match ch {
                '\\' if in_string => escaped = true,
                '"' => in_string = !in_string,
                '(' if !in_string => depth += 1,
                ')' if !in_string => {
                    depth -= 1;
                    if depth == 0 && idx != trimmed.len() - 1 {
                        wraps_entire_expression = false;
                        break;
                    }
                }
                _ => {}
            }
        }

        if !wraps_entire_expression || depth != 0 {
            return trimmed;
        }

        expr = &trimmed[1..trimmed.len() - 1];
    }
}

fn skip_whitespace(s: &str, mut index: usize) -> usize {
    while let Some(ch) = s[index..].chars().next() {
        if !ch.is_whitespace() {
            break;
        }
        index += ch.len_utf8();
    }
    index
}

fn consume_digits(s: &str) -> usize {
    let mut length = 0;
    for ch in s.chars() {
        if ch.is_ascii_digit() {
            length += ch.len_utf8();
        } else {
            break;
        }
    }
    length
}

fn consume_identifier(s: &str) -> usize {
    let mut length = 0;
    for (idx, ch) in s.char_indices() {
        let valid = if idx == 0 {
            ch == '_' || ch.is_alphabetic()
        } else {
            ch == '_' || ch.is_alphanumeric()
        };
        if !valid {
            break;
        }
        length = idx + ch.len_utf8();
    }
    length
}

fn consume_dotted_identifier_chain(s: &str) -> usize {
    let mut index = consume_identifier(s);
    if index == 0 {
        return 0;
    }

    loop {
        let after_dot = skip_whitespace(s, index);
        if after_dot >= s.len() || !s[after_dot..].starts_with('.') {
            return index;
        }

        let property_start = skip_whitespace(s, after_dot + 1);
        let property_len = consume_identifier(&s[property_start..]);
        if property_len == 0 {
            return index;
        }

        index = property_start + property_len;
    }
}

fn find_string_end(s: &str) -> Option<usize> {
    let mut escaped = false;
    for (idx, ch) in s.char_indices().skip(1) {
        if escaped {
            escaped = false;
            continue;
        }

        match ch {
            '\\' => escaped = true,
            '"' => return Some(idx + 1),
            _ => {}
        }
    }
    None
}

fn find_matching_delimiter_end(
    s: &str,
    start_idx: usize,
    open: char,
    close: char,
) -> Option<usize> {
    let mut depth = 0;
    let mut in_string = false;
    let mut escaped = false;

    for (offset, ch) in s[start_idx..].char_indices() {
        if escaped {
            escaped = false;
            continue;
        }

        match ch {
            '\\' if in_string => escaped = true,
            '"' => in_string = !in_string,
            current if current == open && !in_string => depth += 1,
            current if current == close && !in_string => {
                depth -= 1;
                if depth == 0 {
                    return Some(start_idx + offset + ch.len_utf8());
                }
            }
            _ => {}
        }
    }

    None
}

fn should_parse_as_unary_minus(expr: &str) -> bool {
    expr.starts_with('-') && expr.len() > 1
}

fn find_top_level_binary_operator<'a>(
    expr: &'a str,
    operators: &'a [&'a str],
) -> Option<(usize, &'a str)> {
    let mut depth_curly = 0;
    let mut depth_square = 0;
    let mut depth_paren = 0;
    let mut in_string = false;
    let mut escaped = false;
    let mut last_match = None;

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
                    for operator in operators {
                        if expr[idx..].starts_with(operator)
                            && !is_shift_fragment(expr, idx, operator)
                            && !is_unary_operator_position(expr, idx, operator)
                        {
                            last_match = Some((idx, *operator));
                            break;
                        }
                    }
                }
            }
        }
    }

    last_match
}

fn is_shift_fragment(expr: &str, idx: usize, operator: &str) -> bool {
    matches!(operator, "<" | ">")
        && (expr[idx..].starts_with(&format!("{operator}{operator}"))
            || expr[..idx].ends_with(operator))
}

fn is_unary_operator_position(expr: &str, idx: usize, operator: &str) -> bool {
    if operator != "-" && operator != "+" {
        return false;
    }

    let previous = expr[..idx].chars().rev().find(|ch| !ch.is_whitespace());

    matches!(
        previous,
        None | Some(
            '(' | '['
                | '{'
                | ','
                | ':'
                | '+'
                | '-'
                | '*'
                | '/'
                | '%'
                | '&'
                | '|'
                | '<'
                | '>'
                | '='
                | '!'
        )
    )
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

    #[test]
    fn parse_program_handles_mixed_bracket_and_dot_property_access() {
        let statements = parse_program("set value user.profile[key]\n");
        assert_eq!(statements.len(), 1);

        match &statements[0] {
            Statement::Set { var, value } => {
                assert_eq!(var, "value");
                match value {
                    Expression::PropertyAccess { object, property } => {
                        assert!(matches!(
                            property.as_ref(),
                            Expression::Variable(name) if name == "key"
                        ));
                        match object.as_ref() {
                            Expression::PropertyAccess { object, property } => {
                                assert!(matches!(
                                    property.as_ref(),
                                    Expression::StringLiteral(name) if name == "profile"
                                ));
                                assert!(matches!(
                                    object.as_ref(),
                                    Expression::Variable(name) if name == "user"
                                ));
                            }
                            _ => panic!("Expected nested property access"),
                        }
                    }
                    _ => panic!("Expected property access expression"),
                }
            }
            _ => panic!("Expected set statement"),
        }
    }

    #[test]
    fn parse_program_handles_bracket_property_assignment() {
        let statements = parse_program("set user[\"profile\"][key] \"gold\"\n");
        assert_eq!(statements.len(), 1);

        match &statements[0] {
            Statement::PropertySet {
                object,
                property,
                value,
            } => {
                assert!(matches!(
                    property,
                    Expression::Variable(name) if name == "key"
                ));
                assert!(matches!(
                    value,
                    Expression::StringLiteral(text) if text == "gold"
                ));
                match object {
                    Expression::PropertyAccess { object, property } => {
                        assert!(matches!(
                            property.as_ref(),
                            Expression::StringLiteral(text) if text == "profile"
                        ));
                        assert!(matches!(
                            object.as_ref(),
                            Expression::Variable(name) if name == "user"
                        ));
                    }
                    _ => panic!("Expected property path"),
                }
            }
            _ => panic!("Expected property set statement"),
        }
    }

    #[test]
    fn parse_expression_respects_operator_precedence() {
        let statements = parse_program("set value 1 + 2 * 3\n");
        assert!(matches!(
            &statements[0],
            Statement::Set {
                value:
                    Expression::BinaryOperation {
                        operator,
                        right,
                        ..
                    },
                ..
            } if operator == "+"
                && matches!(
                    right.as_ref(),
                    Expression::BinaryOperation { operator, .. } if operator == "*"
                )
        ));
    }
}
