use crate::parser::ast::Expression;

pub fn object_functions() -> Vec<(&'static str, fn(Vec<Expression>) -> Option<Expression>)> {
    vec![
        ("object_to_string", |args: Vec<Expression>| {
            if args.len() == 1 {
                match &args[0] {
                    Expression::Object(_)
                    | Expression::Array(_)
                    | Expression::Number(_)
                    | Expression::Boolean(_)
                    | Expression::StringLiteral(_)
                    | Expression::Null => {
                        Some(Expression::StringLiteral(object_to_string_impl(&args[0])))
                    }
                    _ => None,
                }
            } else {
                None
            }
        }),
        ("object_keys", |args: Vec<Expression>| {
            if args.len() == 1 {
                if let Expression::Object(properties) = &args[0] {
                    let keys = properties
                        .iter()
                        .map(|(key, _)| Expression::StringLiteral(key.clone()))
                        .collect();
                    Some(Expression::Array(keys))
                } else {
                    None
                }
            } else {
                None
            }
        }),
        ("object_values", |args: Vec<Expression>| {
            if args.len() == 1 {
                if let Expression::Object(properties) = &args[0] {
                    let values = properties.iter().map(|(_, value)| value.clone()).collect();
                    Some(Expression::Array(values))
                } else {
                    None
                }
            } else {
                None
            }
        }),
        ("object_has_property", |args: Vec<Expression>| {
            if args.len() == 2 {
                if let (Expression::Object(properties), Expression::StringLiteral(key)) =
                    (&args[0], &args[1])
                {
                    let has_key = properties.contains_key(key);
                    Some(Expression::Boolean(has_key))
                } else {
                    None
                }
            } else {
                None
            }
        }),
        ("object_merge", |args: Vec<Expression>| {
            if args.len() == 2 {
                if let (Expression::Object(props1), Expression::Object(props2)) =
                    (&args[0], &args[1])
                {
                    let mut result = props1.clone();
                    result.extend(props2.clone());
                    Some(Expression::Object(result))
                } else {
                    None
                }
            } else {
                None
            }
        }),
        ("object_create", |args: Vec<Expression>| {
            if args.len() % 2 == 0 {
                let mut properties = std::collections::HashMap::new();

                for i in (0..args.len()).step_by(2) {
                    if let Expression::StringLiteral(key) = &args[i] {
                        properties.insert(key.clone(), args[i + 1].clone());
                    } else {
                        return None;
                    }
                }

                Some(Expression::Object(properties))
            } else {
                None
            }
        }),
    ]
}

fn object_to_string_impl(expr: &Expression) -> String {
    match expr {
        Expression::Number(n) => n.to_string(),
        Expression::Boolean(b) => b.to_string(),
        Expression::StringLiteral(s) => format!("\"{}\"", s),
        Expression::Null => "null".to_string(),
        Expression::Array(arr) => {
            let elements: Vec<String> = arr.iter().map(|e| object_to_string_impl(e)).collect();
            format!("[{}]", elements.join(", "))
        }
        Expression::Object(properties) => {
            let elements: Vec<String> = properties
                .iter()
                .map(|(key, value)| format!("\"{}\": {}", key, object_to_string_impl(value)))
                .collect();
            format!("{{{}}}", elements.join(", "))
        }
        Expression::Variable(_) => "\"<variable>\"".to_string(),
        Expression::FunctionCall { .. } => "\"<function call>\"".to_string(),
        Expression::Comparison { .. } => "\"<comparison>\"".to_string(),
        Expression::PropertyAccess { .. } => "\"<property access>\"".to_string(),
        Expression::StringInterpolation { .. } => "\"<string interpolation>\"".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::object_functions;
    use crate::parser::ast::Expression;

    fn object_fn(name: &str) -> fn(Vec<Expression>) -> Option<Expression> {
        object_functions()
            .into_iter()
            .find(|(n, _)| *n == name)
            .map(|(_, f)| f)
            .expect("missing object function")
    }

    #[test]
    fn create_has_property_and_merge_work() {
        let create = object_fn("object_create");
        let has_property = object_fn("object_has_property");
        let merge = object_fn("object_merge");

        let obj_a = create(vec![
            Expression::StringLiteral("a".to_string()),
            Expression::Number(1),
        ])
        .expect("object_create should return object");
        let obj_b = create(vec![
            Expression::StringLiteral("b".to_string()),
            Expression::Number(2),
        ])
        .expect("object_create should return object");
        let merged = merge(vec![obj_a, obj_b]).expect("object_merge should return object");

        assert!(matches!(
            has_property(vec![
                merged.clone(),
                Expression::StringLiteral("a".to_string())
            ]),
            Some(Expression::Boolean(true))
        ));
        assert!(matches!(
            has_property(vec![merged, Expression::StringLiteral("b".to_string())]),
            Some(Expression::Boolean(true))
        ));
    }

    #[test]
    fn keys_and_values_return_arrays() {
        let create = object_fn("object_create");
        let keys = object_fn("object_keys");
        let values = object_fn("object_values");

        let obj = create(vec![
            Expression::StringLiteral("a".to_string()),
            Expression::Number(1),
            Expression::StringLiteral("b".to_string()),
            Expression::Number(2),
        ])
        .expect("object_create should return object");

        assert!(matches!(
            keys(vec![obj.clone()]),
            Some(Expression::Array(items)) if items.len() == 2
        ));
        assert!(matches!(
            values(vec![obj]),
            Some(Expression::Array(items)) if items.len() == 2
        ));
    }
}
