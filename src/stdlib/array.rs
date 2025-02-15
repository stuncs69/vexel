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
