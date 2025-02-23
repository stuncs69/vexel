// src/parser/old

use crate::parser::ast::{Expression, Statement};
use std::collections::VecDeque;

pub(crate) fn parse_program(code: &str) -> Vec<Statement> {
    let mut lines: VecDeque<&str> = code
        .lines()
        .map(|line| line.split('#').next().unwrap_or("").trim())
        .filter(|line| !line.is_empty())
        .collect();
    parse_block(&mut lines)
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
    // just do the same lol
    let name = extract_before(expr, "(").trim().to_string();
    let args = extract_between(expr, "(", ")")
        .split(',')
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

    // If the value is an object, collect multiple lines
    if value_str == "{" {
        let mut object_lines = Vec::new();
        while let Some(line) = lines.pop_front() {
            if line == "}" {
                break;
            }
            object_lines.push(line);
        }
        value_str = format!("{{ {} }}", object_lines.join(" "));
    }

    let value = parse_expression(&value_str);

    if target.contains('.') {
        let mut parts = target.split('.').collect::<Vec<&str>>();
        let property = parts.pop().unwrap().to_string();
        let object_expr = parse_property_access(&target);
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
    Statement::Return {
        expr: parse_expression(&line[7..].trim()),
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
        return Expression::StringLiteral(expr[1..expr.len() - 1].to_string());
    }

    if expr.starts_with('[') && expr.ends_with(']') {
        return parse_array(expr);
    }

    if expr.starts_with('{') && expr.ends_with('}') {
        return parse_object(expr);
    }

    if let Some(operator) = ["==", "!=", "<", ">"].iter().find(|&&op| expr.contains(op)) {
        let parts: Vec<&str> = expr.split(operator).map(str::trim).collect();
        if parts.len() == 2 {
            let left = parse_expression(parts[0]);
            let right = parse_expression(parts[1]);
            return Expression::Comparison {
                left: Box::new(left),
                operator: operator.to_string(),
                right: Box::new(right),
            };
        }
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
    let mut properties = Vec::new();

    for part in content.split(',').map(str::trim).filter(|s| !s.is_empty()) {
        let key_value: Vec<&str> = part.split(':').map(str::trim).collect();
        if key_value.len() != 2 {
            panic!("Invalid object syntax: {}", expr);
        }

        let key = key_value[0].to_string();
        let value = parse_expression(key_value[1]);

        properties.push((key, value));
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
    let condition = parse_expression(&header[6..].strip_suffix(" start").unwrap().trim());
    let body = parse_block(lines);

    Statement::WhileLoop { condition, body }
}

fn parse_array(expr: &str) -> Expression {
    let content = extract_between(expr, "[", "]");
    let elements: Vec<Expression> = content
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(parse_expression)
        .collect();

    Expression::Array(elements)
}

fn parse_function_call(expr: &str) -> Expression {
    let name = extract_before(expr, "(").trim().to_string();
    let args = extract_between(expr, "(", ")")
        .split(',')
        .map(|arg| parse_expression(arg.trim()))
        .collect();

    Expression::FunctionCall { name, args }
}

fn extract_between<'a>(s: &'a str, start: &str, end: &str) -> &'a str {
    s.split(start)
        .nth(1)
        .unwrap_or("")
        .split(end)
        .next()
        .unwrap_or("")
}

fn extract_before<'a>(s: &'a str, delimiter: &str) -> &'a str {
    s.split(delimiter).next().unwrap_or(s)
}
