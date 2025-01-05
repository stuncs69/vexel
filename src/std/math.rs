use crate::parser::ast::Expression;

pub fn math_functions() -> Vec<(&'static str, fn(Vec<Expression>) -> Option<Expression>)> {
    vec![
        ("math_add", |args: Vec<Expression>| {
            if args.len() == 2 {
                match (&args[0], &args[1]) {
                    (Expression::Number(a), Expression::Number(b)) => Some(Expression::Number(a + b)),
                    _ => None,
                }
            } else {
                None
            }
        }),
        ("math_subtract", |args: Vec<Expression>| {
            if args.len() == 2 {
                match (&args[0], &args[1]) {
                    (Expression::Number(a), Expression::Number(b)) => Some(Expression::Number(a - b)),
                    _ => None,
                }
            } else {
                None
            }
        }),
    ]
}