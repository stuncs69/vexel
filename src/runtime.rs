use crate::parser::ast::{Expression, Statement};
use crate::std::get_all_native_functions;
use std::collections::HashMap;

pub struct Runtime {
    variables: HashMap<String, Expression>,
    functions: HashMap<String, (Vec<String>, Vec<Statement>, bool)>,
    native_functions: HashMap<String, fn(Vec<Expression>) -> Option<Expression>>,
}

impl Runtime {
    pub(crate) fn new() -> Self {
        let mut runtime = Runtime {
            variables: HashMap::new(),
            functions: HashMap::new(),
            native_functions: HashMap::new(),
        };

        runtime.register_native_functions();
        runtime
    }

    fn register_native_functions(&mut self) {
        for (name, func) in get_all_native_functions() {
            self.native_functions.insert(name.to_string(), func);
        }
    }

    pub(crate) fn execute(&mut self, statements: Vec<Statement>) {
        for statement in statements {
            match statement {
                Statement::Set { var, value } => {
                    self.variables.insert(var, value);
                }
                Statement::Function {
                    name,
                    params,
                    body,
                    exported: _,
                } => {
                    self.functions.insert(name, (params, body, false));
                }
                Statement::FunctionCall { name, args } => {
                    if let Some(value) =
                        self.evaluate_expression(Expression::FunctionCall { name, args })
                    {
                        match value {
                            Expression::Number(n) => println!("{}", n),
                            Expression::Boolean(b) => println!("{}", b),
                            Expression::StringLiteral(s) => println!("{}", s),
                            _ => (),
                        }
                    }
                }
                Statement::Print { expr } => {
                    if let Some(value) = self.evaluate_expression(expr) {
                        match value {
                            Expression::Number(n) => println!("{}", n),
                            Expression::Boolean(b) => println!("{}", b),
                            Expression::StringLiteral(s) => println!("{}", s),
                            Expression::Variable(name) => {
                                if let Some(val) = self.variables.get(&name) {
                                    match val {
                                        Expression::Number(n) => println!("{}", n),
                                        Expression::Boolean(b) => println!("{}", b),
                                        Expression::StringLiteral(s) => println!("{}", s),
                                        Expression::FunctionCall { name, args } => {
                                            if let Some(val) =
                                                self.evaluate_expression(Expression::FunctionCall {
                                                    name: name.clone(),
                                                    args: args.clone(),
                                                })
                                            {
                                                match val {
                                                    Expression::Number(n) => println!("{}", n),
                                                    Expression::Boolean(b) => println!("{}", b),
                                                    Expression::StringLiteral(s) => {
                                                        println!("{}", s)
                                                    }
                                                    _ => (),
                                                }
                                            }
                                        }
                                        _ => (),
                                    }
                                }
                            }
                            Expression::FunctionCall { name, args } => {
                                if let Some(val) =
                                    self.evaluate_expression(Expression::FunctionCall {
                                        name: name.clone(),
                                        args: args.clone(),
                                    })
                                {
                                    match val {
                                        Expression::Number(n) => println!("{}", n),
                                        Expression::Boolean(b) => println!("{}", b),
                                        Expression::StringLiteral(s) => println!("{}", s),
                                        _ => (),
                                    }
                                }
                            }
                            _ => (),
                        }
                    }
                }
                Statement::Return { expr } => {}
                Statement::If { condition, body } => {
                    if let Expression::Boolean(true) = self
                        .evaluate_expression(condition)
                        .unwrap_or(Expression::Boolean(false))
                    {
                        self.execute(body);
                    }
                }
            }
        }
    }

    fn evaluate_expression(&self, expr: Expression) -> Option<Expression> {
        match expr {
            Expression::Number(n) => Some(Expression::Number(n)),
            Expression::Boolean(b) => Some(Expression::Boolean(b)),
            Expression::StringLiteral(s) => Some(Expression::StringLiteral(s)),
            Expression::Variable(name) => self.variables.get(&name).cloned(),
            Expression::FunctionCall { name, args } => {
                if let Some(native_func) = self.native_functions.get(&name) {
                    return native_func(args);
                }

                if let Some((params, body, _)) = self.functions.get(&name) {
                    if params.len() == args.len() {
                        let mut local_vars = HashMap::new();
                        for (param, arg) in params.iter().zip(args.into_iter()) {
                            if let Some(value) = self.evaluate_expression(arg) {
                                local_vars.insert(param.clone(), value);
                            }
                        }
                        let mut nested_runtime = Runtime {
                            variables: local_vars,
                            functions: self.functions.clone(),
                            native_functions: self.native_functions.clone(),
                        };
                        nested_runtime.execute(body.clone());
                        None
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            Expression::Comparison {
                left,
                operator,
                right,
            } => {
                let left_val = self.evaluate_expression(*left);
                let right_val = self.evaluate_expression(*right);

                match (left_val, right_val) {
                    (Some(Expression::Number(l)), Some(Expression::Number(r))) => {
                        match operator.as_str() {
                            "==" => Some(Expression::Boolean(l == r)),
                            "!=" => Some(Expression::Boolean(l != r)),
                            ">" => Some(Expression::Boolean(l > r)),
                            "<" => Some(Expression::Boolean(l < r)),
                            ">=" => Some(Expression::Boolean(l >= r)),
                            "<=" => Some(Expression::Boolean(l <= r)),
                            _ => None,
                        }
                    }
                    _ => None,
                }
            }
        }
    }
}
