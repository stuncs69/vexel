use crate::parser::ast::Expression;
use serde_json::Value;

pub fn json_functions() -> Vec<(&'static str, fn(Vec<Expression>) -> Option<Expression>)> {
    vec![
        ("json_parse", |args: Vec<Expression>| {
            if args.len() == 1 {
                if let Expression::StringLiteral(json) = &args[0] {
                    match serde_json::from_str::<Value>(json) {
                        Ok(value) => value_to_expression(&value).map(|e| e),
                        Err(_) => None,
                    }
                } else {
                    None
                }
            } else {
                None
            }
        }),
        ("json_stringify", |args: Vec<Expression>| {
            if args.len() == 1 {
                match expression_to_value(&args[0]) {
                    Some(value) => serde_json::to_string(&value)
                        .ok()
                        .map(|s| Expression::StringLiteral(s)),
                    None => None,
                }
            } else {
                None
            }
        }),
    ]
}

fn value_to_expression(value: &Value) -> Option<Expression> {
    match value {
        Value::Null => Some(Expression::Null),
        Value::Bool(b) => Some(Expression::Boolean(*b)),
        Value::Number(n) => n.as_i64().map(|i| Expression::Number(i as i32)),
        Value::String(s) => Some(Expression::StringLiteral(s.clone())),
        Value::Array(arr) => {
            let mut elements = Vec::new();
            for v in arr {
                elements.push(value_to_expression(v)?);
            }
            Some(Expression::Array(elements))
        }
        Value::Object(map) => {
            let mut props = std::collections::HashMap::new();
            for (k, v) in map {
                props.insert(k.clone(), value_to_expression(v)?);
            }
            Some(Expression::Object(props))
        }
    }
}

fn expression_to_value(expr: &Expression) -> Option<Value> {
    match expr {
        Expression::Null => Some(Value::Null),
        Expression::Boolean(b) => Some(Value::Bool(*b)),
        Expression::Number(n) => Some(Value::Number((*n as i64).into())),
        Expression::StringLiteral(s) => Some(Value::String(s.clone())),
        Expression::Array(arr) => {
            let mut vec = Vec::new();
            for e in arr {
                vec.push(expression_to_value(e)?);
            }
            Some(Value::Array(vec))
        }
        Expression::Object(props) => {
            let mut map = serde_json::Map::new();
            for (k, v) in props {
                map.insert(k.clone(), expression_to_value(v)?);
            }
            Some(Value::Object(map))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::json_functions;
    use crate::parser::ast::Expression;
    use std::collections::HashMap;

    fn json_fn(name: &str) -> fn(Vec<Expression>) -> Option<Expression> {
        json_functions()
            .into_iter()
            .find(|(n, _)| *n == name)
            .map(|(_, f)| f)
            .expect("missing json function")
    }

    #[test]
    fn parse_and_stringify_round_trip() {
        let parse = json_fn("json_parse");
        let stringify = json_fn("json_stringify");

        let parsed = parse(vec![Expression::StringLiteral(
            "{\"x\":1,\"ok\":true}".to_string(),
        )])
        .expect("json_parse should return object");
        assert!(matches!(parsed, Expression::Object(_)));

        let mut obj = HashMap::new();
        obj.insert("x".to_string(), Expression::Number(1));
        obj.insert("ok".to_string(), Expression::Boolean(true));
        let serialized = stringify(vec![Expression::Object(obj)])
            .expect("json_stringify should return string literal");
        assert!(matches!(
            serialized,
            Expression::StringLiteral(s) if s.contains("\"x\":1") && s.contains("\"ok\":true")
        ));
    }

    #[test]
    fn parse_invalid_json_returns_none() {
        let parse = json_fn("json_parse");
        let result = parse(vec![Expression::StringLiteral("{invalid}".to_string())]);
        assert!(result.is_none());
    }
}
