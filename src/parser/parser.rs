// src/parser/old

use std::collections::VecDeque;
use crate::parser::ast::{Statement, Expression};

pub(crate) fn parse_program(code: &str) -> Vec<Statement> {
    let mut lines: VecDeque<&str> = code.lines().map(str::trim).collect();
    parse_block(&mut lines)
}

fn parse_block(lines: &mut VecDeque<&str>) -> Vec<Statement> {
    let mut statements = Vec::new();

    while let Some(line) = lines.pop_front() {
        match line.split_whitespace().next() {
            Some("set") => statements.push(parse_set_statement(line)),
            Some("function") | Some("export") => statements.push(parse_function(lines, line)),
            Some("if") => statements.push(parse_if_statement(lines, line)),
            Some("print") => statements.push(parse_print_statement(line)),
            Some("return") => statements.push(parse_return_statement(line)),
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

fn parse_set_statement(line: &str) -> Statement {
    let parts: Vec<&str> = line.splitn(3, ' ').collect();
    if parts.len() != 3 {
        panic!("Invalid set statement: {}", line);
    }

    let var = parts[1].to_string();
    let value = parse_expression(parts[2]);

    Statement::Set { var, value }
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

    Statement::Function { name, params, body, exported }
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
    if let Ok(num) = expr.parse::<i32>() {
        return Expression::Number(num);
    }

    if expr == "true" || expr == "false" {
        return Expression::Boolean(expr == "true");
    }

    if expr.starts_with('"') && expr.ends_with('"') {
        return Expression::StringLiteral(expr[1..expr.len() - 1].to_string());
    }

    if let Some(operator) = ["==", "!=", "<", ">"].iter().find(|&&op| expr.contains(op)) {
        let parts: Vec<&str> = expr.split(operator).map(str::trim).collect();
        if parts.len() == 2 {
            let left = parse_expression(parts[0]);
            let right = parse_expression(extract_before(parts[1], " start"));
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

    Expression::Variable(expr.to_string())
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
    s.split(start).nth(1).unwrap_or("").split(end).next().unwrap_or("")
}

fn extract_before<'a>(s: &'a str, delimiter: &str) -> &'a str {
    s.split(delimiter).next().unwrap_or(s)
}
