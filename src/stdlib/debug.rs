use crate::parser::ast::Expression;

pub fn debug_functions() -> Vec<(&'static str, fn(Vec<Expression>) -> Option<Expression>)> {
    vec![
        ("dump_type", |args: Vec<Expression>| {
            println!("{:?}", args);
            if args.len() == 1 {
                println!("{:?}", args[0]);
                None
            } else {
                None
            }
        }),
        ("dump", |args: Vec<Expression>| {
            if let Some(value) = args.first() {
                println!("{:?}", value);
            }
            None
        }),
        ("assert_equal", |args: Vec<Expression>| {
            if args.len() == 2 {
                let result = match (&args[0], &args[1]) {
                    (Expression::Number(a), Expression::Number(b)) => a == b,
                    (Expression::Boolean(a), Expression::Boolean(b)) => a == b,
                    (Expression::StringLiteral(a), Expression::StringLiteral(b)) => a == b,
                    (Expression::Null, Expression::Null) => true,
                    _ => false,
                };
                if !result {
                    println!("Assertion failed: {:?} != {:?}", args[0], args[1]);
                }
            }
            None
        }),
    ]
}

#[cfg(test)]
mod tests {
    use super::debug_functions;

    #[test]
    fn dump_with_no_arguments_does_not_panic() {
        let dump = debug_functions()
            .into_iter()
            .find(|(name, _)| *name == "dump")
            .map(|(_, f)| f)
            .expect("missing dump function");

        let result = dump(vec![]);
        assert!(result.is_none());
    }
}
