use crate::parser::ast::Expression;
use std::thread;
use std::time::Duration;

pub fn core_functions() -> Vec<(&'static str, fn(Vec<Expression>) -> Option<Expression>)> {
    vec![
        ("sleep", |args: Vec<Expression>| {
            if args.len() == 1 {
                match &args[0] {
                    Expression::Number(duration) => {
                        if *duration == 0 {
                            loop {
                                thread::sleep(Duration::from_secs(1));
                            }
                        } else {
                            thread::sleep(Duration::from_secs(*duration as u64));
                        }
                        Some(Expression::Null)
                    }
                    _ => None,
                }
            } else {
                None
            }
        }),
        ("type_of", |args: Vec<Expression>| {
            if args.len() == 1 {
                let type_name = match &args[0] {
                    Expression::StringLiteral(_) => "string",
                    Expression::Number(_) => "number",
                    Expression::Boolean(_) => "boolean",
                    Expression::Null => "null",
                    _ => "unknown",
                };
                Some(Expression::StringLiteral(type_name.to_string()))
            } else {
                None
            }
        }),
        ("is_null", |args: Vec<Expression>| {
            if args.len() == 1 {
                match &args[0] {
                    Expression::Null => Some(Expression::Boolean(true)),
                    _ => Some(Expression::Boolean(false)),
                }
            } else {
                None
            }
        }),
        ("exec", |args: Vec<Expression>| {
            if args.len() == 1 {
                match &args[0] {
                    Expression::StringLiteral(command) => {
                        match std::process::Command::new(command).output() {
                            Ok(output) => {
                                let stdout = String::from_utf8_lossy(&output.stdout);
                                Some(Expression::StringLiteral(stdout.to_string()))
                            }
                            Err(_) => None,
                        }
                    }
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
    use super::core_functions;
    use crate::parser::ast::Expression;

    #[test]
    fn exec_returns_none_for_missing_command_instead_of_panicking() {
        let exec = core_functions()
            .into_iter()
            .find(|(name, _)| *name == "exec")
            .map(|(_, f)| f)
            .expect("missing exec function");

        let result = exec(vec![Expression::StringLiteral(
            "definitely_not_a_real_command_123".to_string(),
        )]);
        assert!(result.is_none());
    }
}
