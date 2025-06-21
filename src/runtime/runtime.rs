use crate::parser::ast::{Expression, Statement};
use crate::stdlib::get_all_native_functions;
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
                Statement::PropertySet {
                    object,
                    property,
                    value,
                } => {
                    let evaluated_value =
                        self.evaluate_expression(value).unwrap_or(Expression::Null);

                    fn update_property_in_expr(
                        expr: &mut Expression,
                        path: &[String],
                        value: Expression,
                    ) -> bool {
                        if path.is_empty() {
                            return false;
                        }

                        if path.len() == 1 {
                            if let Expression::Object(properties) = expr {
                                for (key, val) in properties.iter_mut() {
                                    if key == &path[0] {
                                        *val = value;
                                        return true;
                                    }
                                }
                                properties.push((path[0].clone(), value));
                                return true;
                            }
                            return false;
                        }

                        if let Expression::Object(properties) = expr {
                            for (key, val) in properties.iter_mut() {
                                if key == &path[0] {
                                    return update_property_in_expr(val, &path[1..], value);
                                }
                            }
                            let mut new_obj = Expression::Object(vec![]);
                            update_property_in_expr(&mut new_obj, &path[1..], value);
                            properties.push((path[0].clone(), new_obj));
                            return true;
                        }
                        false
                    }

                    let mut path = Vec::new();
                    let mut current = &object;
                    let root_var = loop {
                        match current {
                            Expression::Variable(name) => {
                                break name.clone();
                            }
                            Expression::PropertyAccess { object, property } => {
                                path.push(property.clone());
                                current = object;
                            }
                            _ => return, // Invalid property access
                        }
                    };

                    path.reverse();
                    path.push(property);

                    if let Some(mut root_value) = self.variables.remove(&root_var) {
                        update_property_in_expr(&mut root_value, &path, evaluated_value);
                        self.variables.insert(root_var, root_value);
                    } else {
                        let mut new_value = Expression::Object(vec![]);
                        update_property_in_expr(&mut new_value, &path, evaluated_value);
                        self.variables.insert(root_var, new_value);
                    }
                }

                Statement::ForLoop {
                    variable,
                    iterable,
                    body,
                } => {
                    if let Some(Expression::Array(elements)) = self.evaluate_expression(iterable) {
                        for element in elements {
                            self.variables.insert(variable.clone(), element);
                            self.execute(body.clone());
                        }
                    }
                }
                Statement::WhileLoop { condition, body } => {
                    while let Some(Expression::Boolean(true))
                    | Some(Expression::Variable(_))
                    | Some(Expression::Comparison { .. }) =
                        self.evaluate_expression(condition.clone())
                    {
                        self.execute(body.clone());
                    }
                }
                Statement::Set { var, value } => {
                    let evaluated_value =
                        self.evaluate_expression(value).unwrap_or(Expression::Null);
                    self.variables.insert(var, evaluated_value);
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
                        self.print_expression(&value);
                    }
                }
                Statement::Print { expr } => {
                    if let Some(value) = self.evaluate_expression(expr) {
                        self.print_expression(&value);
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

    fn print_expression(&self, expr: &Expression) {
        match expr {
            Expression::Number(n) => println!("{}", n),
            Expression::Boolean(b) => println!("{}", b),
            Expression::StringLiteral(s) => println!("{}", s),
            Expression::Null => println!("null"),
            Expression::Array(arr) => {
                let elements: Vec<String> =
                    arr.iter().map(|e| self.expression_to_string(e)).collect();
                println!("[{}]", elements.join(", "));
            }
            Expression::Object(properties) => {
                let elements: Vec<String> = properties
                    .iter()
                    .map(|(key, value)| format!("{}: {}", key, self.expression_to_string(value)))
                    .collect();
                println!("{{{}}}", elements.join(", "));
            }
            Expression::Variable(name) => {
                if let Some(val) = self.variables.get(name) {
                    self.print_expression(val);
                } else {
                    println!("undefined");
                }
            }
            Expression::FunctionCall { name, args } => {
                if let Some(val) = self.evaluate_expression(Expression::FunctionCall {
                    name: name.clone(),
                    args: args.clone(),
                }) {
                    self.print_expression(&val);
                }
            }
            _ => println!(""),
        }
    }

    fn expression_to_string(&self, expr: &Expression) -> String {
        match expr {
            Expression::Number(n) => n.to_string(),
            Expression::Boolean(b) => b.to_string(),
            Expression::StringLiteral(s) => format!("\"{}\"", s),
            Expression::Null => "null".to_string(),
            Expression::Array(arr) => {
                let elements: Vec<String> =
                    arr.iter().map(|e| self.expression_to_string(e)).collect();
                format!("[{}]", elements.join(", "))
            }
            Expression::Object(properties) => {
                let elements: Vec<String> = properties
                    .iter()
                    .map(|(key, value)| format!("{}: {}", key, self.expression_to_string(value)))
                    .collect();
                format!("{{{}}}", elements.join(", "))
            }
            Expression::Variable(name) => {
                if let Some(val) = self.variables.get(name) {
                    self.expression_to_string(val)
                } else {
                    "undefined".to_string()
                }
            }
            Expression::FunctionCall { name, args } => {
                if let Some(val) = self.evaluate_expression(Expression::FunctionCall {
                    name: name.clone(),
                    args: args.clone(),
                }) {
                    self.expression_to_string(&val)
                } else {
                    "undefined".to_string()
                }
            }
            _ => "".to_string(),
        }
    }

    fn evaluate_expression(&self, expr: Expression) -> Option<Expression> {
        match expr {
            Expression::Array(elements) => {
                let evaluated_elements: Vec<Expression> = elements
                    .into_iter()
                    .filter_map(|e| self.evaluate_expression(e))
                    .collect();
                Some(Expression::Array(evaluated_elements))
            }
            Expression::Object(properties) => {
                let evaluated_properties: Vec<(String, Expression)> = properties
                    .into_iter()
                    .filter_map(|(key, value)| {
                        if let Some(evaluated_value) = self.evaluate_expression(value) {
                            Some((key, evaluated_value))
                        } else {
                            None
                        }
                    })
                    .collect();
                Some(Expression::Object(evaluated_properties))
            }
            Expression::Number(n) => Some(Expression::Number(n)),
            Expression::Null => Some(Expression::Null),
            Expression::Boolean(b) => Some(Expression::Boolean(b)),
            Expression::StringLiteral(s) => Some(Expression::StringLiteral(s)),
            Expression::Variable(name) => {
                if let Some(Expression::Object(properties)) = self.variables.get(&name) {
                    let mut new_properties = Vec::new();
                    for (key, value) in properties {
                        new_properties.push((key.clone(), value.clone()));
                    }
                    return Some(Expression::Object(new_properties));
                }
                self.variables.get(&name).cloned()
            }
            Expression::FunctionCall { name, args } => {
                let evaluated_args: Vec<Expression> = args
                    .into_iter()
                    .map(|arg| self.evaluate_expression(arg).unwrap_or(Expression::Null))
                    .collect();

                if let Some(native_func) = self.native_functions.get(&name) {
                    native_func(evaluated_args)
                } else if let Some((params, body, _)) = self.functions.get(&name) {
                    if params.len() == evaluated_args.len() {
                        let mut local_vars = HashMap::new();
                        for (param, arg) in params.iter().zip(evaluated_args.into_iter()) {
                            local_vars.insert(param.clone(), arg);
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
                    (Some(Expression::Boolean(l)), Some(Expression::Boolean(r))) => {
                        match operator.as_str() {
                            "==" => Some(Expression::Boolean(l == r)),
                            "!=" => Some(Expression::Boolean(l != r)),
                            _ => None,
                        }
                    }
                    _ => None,
                }
            }
            Expression::PropertyAccess { object, property } => {
                if let Some(evaluated_object) = self.evaluate_expression(*object) {
                    match evaluated_object {
                        Expression::Object(properties) => {
                            for (key, value) in properties {
                                if key == property {
                                    return Some(value);
                                }
                            }
                            None
                        }
                        _ => None,
                    }
                } else {
                    None
                }
            }
        }
    }
}
