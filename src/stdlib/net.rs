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
