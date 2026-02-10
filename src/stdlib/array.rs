use crate::parser::ast::Expression;

pub fn array_functions() -> Vec<(&'static str, fn(Vec<Expression>) -> Option<Expression>)> {
    vec![
        ("array_push", array_push),
        ("array_pop", array_pop),
        ("array_length", array_length),
        ("array_get", array_get),
        ("array_set", array_set),
        ("array_slice", array_slice),
        ("array_join", array_join),
        ("array_to_string", array_to_string),
        ("array_range", array_range),
    ]
}

fn array_push(args: Vec<Expression>) -> Option<Expression> {
    if args.len() < 2 {
        return None;
    }
    if let Expression::Array(mut arr) = args[0].clone() {
        arr.extend_from_slice(&args[1..]);
        Some(Expression::Array(arr))
    } else {
        None
    }
}

fn array_pop(args: Vec<Expression>) -> Option<Expression> {
    if args.len() != 1 {
        return None;
    }
    if let Expression::Array(mut arr) = args[0].clone() {
        arr.pop().map(|item| Expression::Array(vec![item]))
    } else {
        None
    }
}

fn array_length(args: Vec<Expression>) -> Option<Expression> {
    if args.len() != 1 {
        return None;
    }
    if let Expression::Array(arr) = &args[0] {
        Some(Expression::Number(arr.len() as i32))
    } else {
        None
    }
}

fn array_get(args: Vec<Expression>) -> Option<Expression> {
    if args.len() != 2 {
        return None;
    }
    if let (Expression::Array(arr), Expression::Number(index)) = (&args[0], &args[1]) {
        arr.get(*index as usize).cloned()
    } else {
        None
    }
}

fn array_set(args: Vec<Expression>) -> Option<Expression> {
    if args.len() != 3 {
        return None;
    }
    if let (Expression::Array(mut arr), Expression::Number(index), value) =
        (args[0].clone(), &args[1], &args[2])
    {
        if (*index as usize) < arr.len() {
            arr[*index as usize] = value.clone();
            Some(Expression::Array(arr))
        } else {
            None
        }
    } else {
        None
    }
}

fn array_slice(args: Vec<Expression>) -> Option<Expression> {
    if args.len() != 3 {
        return None;
    }
    if let (Expression::Array(arr), Expression::Number(start), Expression::Number(end)) =
        (&args[0], &args[1], &args[2])
    {
        let start = *start as usize;
        let end = *end as usize;
        if start <= end && end <= arr.len() {
            Some(Expression::Array(arr[start..end].to_vec()))
        } else {
            None
        }
    } else {
        None
    }
}

fn array_join(args: Vec<Expression>) -> Option<Expression> {
    if args.len() != 2 {
        return None;
    }
    if let (Expression::Array(arr), Expression::StringLiteral(separator)) = (&args[0], &args[1]) {
        let joined = arr
            .iter()
            .map(|expr| match expr {
                Expression::StringLiteral(s) => s.clone(),
                Expression::Number(n) => n.to_string(),
                Expression::Boolean(b) => b.to_string(),
                Expression::Null => "null".to_string(),
                _ => "".to_string(),
            })
            .collect::<Vec<String>>()
            .join(separator);
        Some(Expression::StringLiteral(joined))
    } else {
        None
    }
}

fn array_to_string(args: Vec<Expression>) -> Option<Expression> {
    if args.len() != 1 {
        return None;
    }
    if let Expression::Array(arr) = &args[0] {
        let elements = arr
            .iter()
            .map(|e| match e {
                Expression::Number(n) => n.to_string(),
                Expression::Boolean(b) => b.to_string(),
                Expression::StringLiteral(s) => format!("{}", s),
                Expression::PropertyAccess { object, property } => format!(""),
                Expression::StringInterpolation { .. } => "<string interpolation>".to_string(),
                Expression::Object(properties) => format!(""),
                Expression::Null => "null".to_string(),
                Expression::Array(_) => "[...]".to_string(),
                Expression::FunctionCall { name, args } => {
                    format!("{}({:?})", name, args)
                }
                Expression::Variable(name) => name.clone(),
                Expression::Comparison {
                    left,
                    operator,
                    right,
                } => {
                    format!("")
                }
            })
            .collect::<Vec<String>>()
            .join("");
        Some(Expression::StringLiteral(format!("{}", elements)))
    } else {
        None
    }
}

fn array_range(args: Vec<Expression>) -> Option<Expression> {
    if args.len() != 1 {
        return None;
    }

    if let Expression::Number(n) = &args[0] {
        let mut arr = Vec::new();
        for i in 0..*n {
            arr.push(Expression::Number(i));
        }
        Some(Expression::Array(arr))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::array_functions;
    use crate::parser::ast::Expression;

    fn array_fn(name: &str) -> fn(Vec<Expression>) -> Option<Expression> {
        array_functions()
            .into_iter()
            .find(|(n, _)| *n == name)
            .map(|(_, f)| f)
            .expect("missing array function")
    }

    #[test]
    fn push_get_set_and_length_work_together() {
        let push = array_fn("array_push");
        let get = array_fn("array_get");
        let set = array_fn("array_set");
        let length = array_fn("array_length");

        let original = Expression::Array(vec![Expression::Number(1), Expression::Number(2)]);
        let pushed =
            push(vec![original, Expression::Number(3)]).expect("array_push should return array");
        assert!(matches!(
            length(vec![pushed.clone()]),
            Some(Expression::Number(3))
        ));
        assert!(matches!(
            get(vec![pushed.clone(), Expression::Number(2)]),
            Some(Expression::Number(3))
        ));

        let updated = set(vec![pushed, Expression::Number(1), Expression::Number(9)])
            .expect("array_set should return array");
        assert!(matches!(
            get(vec![updated, Expression::Number(1)]),
            Some(Expression::Number(9))
        ));
    }

    #[test]
    fn range_slice_pop_and_join_work() {
        let range = array_fn("array_range");
        let slice = array_fn("array_slice");
        let pop = array_fn("array_pop");
        let join = array_fn("array_join");

        let values = range(vec![Expression::Number(5)]).expect("array_range should return array");
        let sliced = slice(vec![values, Expression::Number(1), Expression::Number(4)])
            .expect("array_slice should return array");
        assert!(matches!(
            join(vec![
                sliced.clone(),
                Expression::StringLiteral(",".to_string())
            ]),
            Some(Expression::StringLiteral(s)) if s == "1,2,3"
        ));
        assert!(matches!(
            pop(vec![sliced]),
            Some(Expression::Array(items)) if matches!(items.as_slice(), [Expression::Number(3)])
        ));
    }
}
