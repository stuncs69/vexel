use crate::parser::ast::Expression;
use reqwest::blocking::Client;

pub fn http_functions() -> Vec<(&'static str, fn(Vec<Expression>) -> Option<Expression>)> {
    vec![
        ("http_get", |args: Vec<Expression>| {
            if args.len() == 1 {
                if let Expression::StringLiteral(url) = &args[0] {
                    let client = Client::new();
                    match client.get(url).send() {
                        Ok(response) => match response.text() {
                            Ok(body) => Some(Expression::StringLiteral(body)),
                            Err(_) => None,
                        },
                        Err(_) => None,
                    }
                } else {
                    None
                }
            } else {
                None
            }
        }),
        ("http_post", |args: Vec<Expression>| {
            if args.len() == 2 {
                if let (Expression::StringLiteral(url), Expression::StringLiteral(body)) =
                    (&args[0], &args[1])
                {
                    let client = Client::new();
                    match client.post(url).body(body.clone()).send() {
                        Ok(response) => match response.text() {
                            Ok(body) => Some(Expression::StringLiteral(body)),
                            Err(_) => None,
                        },
                        Err(_) => None,
                    }
                } else {
                    None
                }
            } else {
                None
            }
        }),
        ("http_put", |args: Vec<Expression>| {
            if args.len() == 2 {
                if let (Expression::StringLiteral(url), Expression::StringLiteral(body)) =
                    (&args[0], &args[1])
                {
                    let client = Client::new();
                    match client.put(url).body(body.clone()).send() {
                        Ok(response) => match response.text() {
                            Ok(body) => Some(Expression::StringLiteral(body)),
                            Err(_) => None,
                        },
                        Err(_) => None,
                    }
                } else {
                    None
                }
            } else {
                None
            }
        }),
        ("http_delete", |args: Vec<Expression>| {
            if args.len() == 1 {
                if let Expression::StringLiteral(url) = &args[0] {
                    let client = Client::new();
                    match client.delete(url).send() {
                        Ok(response) => match response.text() {
                            Ok(body) => Some(Expression::StringLiteral(body)),
                            Err(_) => None,
                        },
                        Err(_) => None,
                    }
                } else {
                    None
                }
            } else {
                None
            }
        }),
    ]
}

#[cfg(test)]
mod tests {
    use super::http_functions;
    use crate::parser::ast::Expression;

    fn http_fn(name: &str) -> fn(Vec<Expression>) -> Option<Expression> {
        http_functions()
            .into_iter()
            .find(|(n, _)| *n == name)
            .map(|(_, f)| f)
            .expect("missing http function")
    }

    #[test]
    fn malformed_url_returns_none() {
        let get = http_fn("http_get");
        let result = get(vec![Expression::StringLiteral("not_a_url".to_string())]);
        assert!(result.is_none());
    }

    #[test]
    fn wrong_argument_types_return_none() {
        let post = http_fn("http_post");
        let put = http_fn("http_put");
        let delete = http_fn("http_delete");

        assert!(post(vec![Expression::Number(1), Expression::Number(2)]).is_none());
        assert!(put(vec![Expression::Number(1), Expression::Number(2)]).is_none());
        assert!(delete(vec![Expression::Number(1)]).is_none());
    }
}
