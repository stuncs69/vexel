use crate::parser::ast::Expression;

pub fn string_functions() -> Vec<(&'static str, fn(Vec<Expression>) -> Option<Expression>)> {
    vec![
        ("string_length", |args: Vec<Expression>| {
            if args.len() == 1 {
                match &args[0] {
                    Expression::StringLiteral(s) => Some(Expression::Number(s.len() as i32)),
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
                            let char_count = s.chars().count();
                            if start > char_count {
                                None
                            } else if let Some(end) = start.checked_add(length) {
                                if end > char_count {
                                    None
                                } else {
                                    Some(Expression::StringLiteral(
                                        s.chars().skip(start).take(length).collect(),
                                    ))
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
