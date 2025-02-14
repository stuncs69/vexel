use crate::parser::ast::Expression;

pub fn math_functions() -> Vec<(&'static str, fn(Vec<Expression>) -> Option<Expression>)> {
    vec![
        ("math_add", |args: Vec<Expression>| {
            if args.len() == 2 {
                match (&args[0], &args[1]) {
                    (Expression::Number(a), Expression::Number(b)) => {
                        Some(Expression::Number(a + b))
                    }
                    _ => None,
                }
            } else {
                None
            }
        }),
        ("math_subtract", |args: Vec<Expression>| {
            if args.len() == 2 {
                match (&args[0], &args[1]) {
                    (Expression::Number(a), Expression::Number(b)) => {
                        Some(Expression::Number(a - b))
                    }
                    _ => None,
                }
            } else {
                None
            }
        }),
        ("math_multiply", |args: Vec<Expression>| {
            if args.len() == 2 {
                match (&args[0], &args[1]) {
                    (Expression::Number(a), Expression::Number(b)) => {
                        Some(Expression::Number(a * b))
                    }
                    _ => None,
                }
            } else {
                None
            }
        }),
        ("math_divide", |args: Vec<Expression>| {
            if args.len() == 2 {
                match (&args[0], &args[1]) {
                    (Expression::Number(a), Expression::Number(b)) if *b != 0 => {
                        Some(Expression::Number(a / b))
                    }
                    _ => None,
                }
            } else {
                None
            }
        }),
        ("math_power", |args: Vec<Expression>| {
            if args.len() == 2 {
                match (&args[0], &args[1]) {
                    (Expression::Number(a), Expression::Number(b)) if *b >= 0 => {
                        Some(Expression::Number(a.pow(*b as u32)))
                    }
                    _ => None,
                }
            } else {
                None
            }
        }),
        ("math_sqrt", |args: Vec<Expression>| {
            if args.len() == 1 {
                match &args[0] {
                    Expression::Number(a) if *a >= 0 => {
                        Some(Expression::Number(((*a as f64).sqrt()) as i32))
                    }
                    _ => None,
                }
            } else {
                None
            }
        }),
        ("math_abs", |args: Vec<Expression>| {
            if args.len() == 1 {
                match &args[0] {
                    Expression::Number(a) => Some(Expression::Number(a.abs())),
                    _ => None,
                }
            } else {
                None
            }
        }),
    ]
}
