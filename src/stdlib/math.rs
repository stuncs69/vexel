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

#[cfg(test)]
mod tests {
    use super::math_functions;
    use crate::parser::ast::Expression;

    fn math_fn(name: &str) -> fn(Vec<Expression>) -> Option<Expression> {
        math_functions()
            .into_iter()
            .find(|(n, _)| *n == name)
            .map(|(_, f)| f)
            .expect("missing math function")
    }

    #[test]
    fn add_and_multiply_return_expected_values() {
        let add = math_fn("math_add");
        let multiply = math_fn("math_multiply");

        assert!(matches!(
            add(vec![Expression::Number(4), Expression::Number(6)]),
            Some(Expression::Number(10))
        ));
        assert!(matches!(
            multiply(vec![Expression::Number(3), Expression::Number(7)]),
            Some(Expression::Number(21))
        ));
    }

    #[test]
    fn divide_by_zero_returns_none() {
        let divide = math_fn("math_divide");
        assert!(divide(vec![Expression::Number(8), Expression::Number(0)]).is_none());
    }

    #[test]
    fn power_and_abs_return_expected_values() {
        let power = math_fn("math_power");
        let abs = math_fn("math_abs");

        assert!(matches!(
            power(vec![Expression::Number(2), Expression::Number(5)]),
            Some(Expression::Number(32))
        ));
        assert!(matches!(
            abs(vec![Expression::Number(-9)]),
            Some(Expression::Number(9))
        ));
    }
}
