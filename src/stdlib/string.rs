use crate::parser::ast::Expression;

pub fn string_functions() -> Vec<(&'static str, fn(Vec<Expression>) -> Option<Expression>)> {
    vec![
        ("string_length", |args: Vec<Expression>| {
            if args.len() == 1 {
                match &args[0] {
                    Expression::StringLiteral(s) => {
                        Some(Expression::Number(s.chars().count() as i32))
                    }
                    _ => None,
                }
            } else {
                None
            }
        }),
        ("string_concat", |args: Vec<Expression>| {
            if args.len() >= 2 {
                let mut result = String::new();
                for arg in args {
                    match arg {
                        Expression::StringLiteral(s) => result.push_str(&s),
                        _ => return None,
                    }
                }
                Some(Expression::StringLiteral(result))
            } else {
                None
            }
        }),
        ("string_from_number", |args: Vec<Expression>| {
            if args.len() == 1 {
                match &args[0] {
                    Expression::Number(n) => Some(Expression::StringLiteral(n.to_string())),
                    _ => None,
                }
            } else {
                None
            }
        }),
        ("number_from_string", |args: Vec<Expression>| {
            if args.len() == 1 {
                match &args[0] {
                    Expression::StringLiteral(s) => s.parse::<i32>().ok().map(Expression::Number),
                    _ => None,
                }
            } else {
                None
            }
        }),
        ("string_substring", |args: Vec<Expression>| {
            if args.len() == 3 {
                match (&args[0], &args[1], &args[2]) {
                    (
                        Expression::StringLiteral(s),
                        Expression::Number(start),
                        Expression::Number(length),
                    ) => {
                        if *start < 0 || *length < 0 {
                            None
                        } else {
                            let start = *start as usize;
                            let length = *length as usize;
                            let chars: Vec<char> = s.chars().collect();
                            if start <= chars.len() {
                                if let Some(end) = start.checked_add(length) {
                                    if end <= chars.len() {
                                        let substr: String = chars[start..end].iter().collect();
                                        Some(Expression::StringLiteral(substr))
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        }
                    }
                    _ => None,
                }
            } else {
                None
            }
        }),
        ("string_contains", |args: Vec<Expression>| {
            if args.len() == 2 {
                match (&args[0], &args[1]) {
                    (Expression::StringLiteral(s), Expression::StringLiteral(sub)) => {
                        Some(Expression::Boolean(s.contains(sub)))
                    }
                    _ => None,
                }
            } else {
                None
            }
        }),
        ("string_replace", |args: Vec<Expression>| {
            if args.len() == 3 {
                match (&args[0], &args[1], &args[2]) {
                    (
                        Expression::StringLiteral(s),
                        Expression::StringLiteral(old),
                        Expression::StringLiteral(new),
                    ) => Some(Expression::StringLiteral(s.replace(old, new))),
                    _ => None,
                }
            } else {
                None
            }
        }),
        ("string_to_upper", |args: Vec<Expression>| {
            if args.len() == 1 {
                match &args[0] {
                    Expression::StringLiteral(s) => {
                        Some(Expression::StringLiteral(s.to_uppercase()))
                    }
                    _ => None,
                }
            } else {
                None
            }
        }),
        ("string_to_lower", |args: Vec<Expression>| {
            if args.len() == 1 {
                match &args[0] {
                    Expression::StringLiteral(s) => {
                        Some(Expression::StringLiteral(s.to_lowercase()))
                    }
                    _ => None,
                }
            } else {
                None
            }
        }),
        ("string_trim", |args: Vec<Expression>| {
            if args.len() == 1 {
                match &args[0] {
                    Expression::StringLiteral(s) => {
                        Some(Expression::StringLiteral(s.trim().to_string()))
                    }
                    _ => None,
                }
            } else {
                None
            }
        }),
        ("string_starts_with", |args: Vec<Expression>| {
            if args.len() == 2 {
                match (&args[0], &args[1]) {
                    (Expression::StringLiteral(s), Expression::StringLiteral(prefix)) => {
                        Some(Expression::Boolean(s.starts_with(prefix)))
                    }
                    _ => None,
                }
            } else {
                None
            }
        }),
        ("string_ends_with", |args: Vec<Expression>| {
            if args.len() == 2 {
                match (&args[0], &args[1]) {
                    (Expression::StringLiteral(s), Expression::StringLiteral(suffix)) => {
                        Some(Expression::Boolean(s.ends_with(suffix)))
                    }
                    _ => None,
                }
            } else {
                None
            }
        }),
    ]
}

#[cfg(test)]
mod tests {
    use super::string_functions;
    use crate::parser::ast::Expression;

    #[test]
    fn string_substring_handles_utf8_without_panicking() {
        let func = string_functions()
            .into_iter()
            .find(|(name, _)| *name == "string_substring")
            .map(|(_, f)| f)
            .expect("missing string_substring function");

        let result = func(vec![
            Expression::StringLiteral("é".to_string()),
            Expression::Number(0),
            Expression::Number(1),
        ]);

        match result {
            Some(Expression::StringLiteral(value)) => assert_eq!(value, "é"),
            _ => panic!("Expected utf8 substring result"),
        }
    }
}
