use crate::parser::ast::{Expression, InterpolationPart, Statement};
use crate::parser::parser::try_parse_program;
use crate::stdlib::get_all_native_functions;
use crate::stdlib::thread;
use rustc_hash::FxHashMap as HashMap;
use std::cell::RefCell;
use std::fs;
use std::rc::Rc;

type FunctionSignature = (Vec<String>, Vec<Statement>, bool);
type FunctionTable = HashMap<String, FunctionSignature>;
type SharedFunctionTable = Rc<RefCell<FunctionTable>>;
type NativeFunction = fn(Vec<Expression>) -> Option<Expression>;
type NativeFunctionTable = Rc<HashMap<String, NativeFunction>>;
type ModuleTable = HashMap<String, FunctionTable>;
type SharedModuleTable = Rc<RefCell<ModuleTable>>;

pub struct Runtime {
    variables: HashMap<String, Expression>,
    functions: SharedFunctionTable,
    native_functions: NativeFunctionTable,
    modules: SharedModuleTable,
    module_cache_by_path: SharedModuleTable,
}

impl Runtime {
    pub(crate) fn new() -> Self {
        let mut runtime = Runtime {
            variables: HashMap::default(),
            functions: Rc::new(RefCell::new(HashMap::default())),
            native_functions: Rc::new(HashMap::default()),
            modules: Rc::new(RefCell::new(HashMap::default())),
            module_cache_by_path: Rc::new(RefCell::new(HashMap::default())),
        };

        runtime.register_native_functions();
        runtime
    }

    fn register_native_functions(&mut self) {
        let mut map = HashMap::default();
        for (name, func) in get_all_native_functions() {
            map.insert(name.to_string(), func);
        }
        self.native_functions = Rc::new(map);
    }

    pub(crate) fn execute(&mut self, statements: &[Statement]) -> Option<Expression> {
        for statement in statements {
            match statement {
                Statement::PropertySet {
                    object,
                    property,
                    value,
                } => {
                    let evaluated_value = self
                        .evaluate_expression(value.clone())
                        .unwrap_or(Expression::Null);

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
                                properties.insert(path[0].clone(), value);
                                return true;
                            }
                            return false;
                        }

                        if let Expression::Object(properties) = expr {
                            if let Some(val) = properties.get_mut(&path[0]) {
                                return update_property_in_expr(val, &path[1..], value);
                            }
                            let mut new_obj = Expression::Object(std::collections::HashMap::new());
                            update_property_in_expr(&mut new_obj, &path[1..], value);
                            properties.insert(path[0].clone(), new_obj);
                            return true;
                        }
                        false
                    }

                    let mut path = Vec::new();
                    let mut current = object;
                    let root_var = loop {
                        match current {
                            Expression::Variable(name) => {
                                break name.clone();
                            }
                            Expression::PropertyAccess { object, property } => {
                                path.push(property.clone());
                                current = object.as_ref();
                            }
                            _ => return None,
                        }
                    };

                    path.reverse();
                    path.push(property.clone());

                    if let Some(mut root_value) = self.variables.remove(&root_var) {
                        update_property_in_expr(&mut root_value, &path, evaluated_value);
                        self.variables.insert(root_var, root_value);
                    } else {
                        let mut new_value = Expression::Object(std::collections::HashMap::new());
                        update_property_in_expr(&mut new_value, &path, evaluated_value);
                        self.variables.insert(root_var, new_value);
                    }
                }

                Statement::ForLoop {
                    variable,
                    iterable,
                    body,
                } => {
                    if let Some(Expression::Array(elements)) =
                        self.evaluate_expression(iterable.clone())
                    {
                        for element in elements {
                            self.variables.insert(variable.clone(), element);
                            if let Some(return_value) = self.execute(body) {
                                return Some(return_value);
                            }
                        }
                    }
                }
                Statement::WhileLoop { condition, body } => {
                    while let Some(Expression::Boolean(true))
                    | Some(Expression::Variable(_))
                    | Some(Expression::Comparison { .. }) =
                        self.evaluate_expression(condition.clone())
                    {
                        if let Some(return_value) = self.execute(body) {
                            return Some(return_value);
                        }
                    }
                }
                Statement::Set { var, value } => {
                    let evaluated_value = self
                        .evaluate_expression(value.clone())
                        .unwrap_or(Expression::Null);
                    self.variables.insert(var.clone(), evaluated_value);
                }
                Statement::Function {
                    name,
                    params,
                    body,
                    exported,
                } => {
                    self.functions
                        .borrow_mut()
                        .insert(name.clone(), (params.clone(), body.clone(), *exported));
                    thread::update_functions_snapshot(self.functions.borrow().clone());
                }
                Statement::FunctionCall { name, args } => {
                    if let Some(value) = self.evaluate_expression(Expression::FunctionCall {
                        name: name.clone(),
                        args: args.clone(),
                    }) {
                        self.print_expression(&value);
                    }
                }
                Statement::Print { expr } => {
                    if let Some(value) = self.evaluate_expression(expr.clone()) {
                        self.print_expression(&value);
                    }
                }
                Statement::Return { expr } => {
                    return self.evaluate_expression(expr.clone());
                }
                Statement::If { condition, body } => {
                    if let Expression::Boolean(true) = self
                        .evaluate_expression(condition.clone())
                        .unwrap_or(Expression::Boolean(false))
                    {
                        if let Some(return_value) = self.execute(body) {
                            return Some(return_value);
                        }
                    }
                }
                Statement::Import {
                    module_name,
                    file_path,
                } => {
                    let resolved_path = if file_path.starts_with("./") {
                        file_path.strip_prefix("./").unwrap().to_string()
                    } else {
                        file_path.clone()
                    };

                    if let Some(cached) = self
                        .module_cache_by_path
                        .borrow()
                        .get(&resolved_path)
                        .cloned()
                    {
                        self.modules
                            .borrow_mut()
                            .insert(module_name.clone(), cached);
                        continue;
                    }

                    match fs::read_to_string(&resolved_path) {
                        Ok(content) => {
                            let module_statements = match try_parse_program(&content) {
                                Ok(stmts) => stmts,
                                Err(e) => {
                                    eprintln!("{}", e);
                                    Vec::new()
                                }
                            };

                            let mut module_runtime = Runtime::new();
                            module_runtime.execute(&module_statements);

                            let funcs_clone = module_runtime.functions.borrow().clone();
                            self.modules
                                .borrow_mut()
                                .insert(module_name.clone(), funcs_clone.clone());
                            self.module_cache_by_path
                                .borrow_mut()
                                .insert(resolved_path, funcs_clone);
                            thread::update_modules_snapshot(self.modules.borrow().clone());
                            thread::update_module_cache_snapshot(
                                self.module_cache_by_path.borrow().clone(),
                            );
                        }
                        Err(e) => {
                            eprintln!("Error loading module from '{}': {}", file_path, e);
                        }
                    }
                }
                Statement::Test { name, body } => {
                    println!("Running test: {}", name);
                    let mut nested_runtime =
                        self.create_nested_runtime(HashMap::default(), self.functions.clone());
                    let _ = nested_runtime.execute(body);
                    println!("Test '{}' finished", name);
                }
            }
        }
        None
    }

    fn render_interpolation(&self, parts: &[InterpolationPart]) -> String {
        let mut rendered = String::new();

        for part in parts {
            match part {
                InterpolationPart::Text(text) => rendered.push_str(text),
                InterpolationPart::Expression(expr) => {
                    if let Some(evaluated) = self.evaluate_expression(expr.clone()) {
                        match evaluated {
                            Expression::StringLiteral(s) => rendered.push_str(&s),
                            Expression::Number(n) => rendered.push_str(&n.to_string()),
                            Expression::Boolean(b) => rendered.push_str(&b.to_string()),
                            Expression::Null => rendered.push_str("null"),
                            _ => rendered.push_str(&self.expression_to_string(&evaluated)),
                        }
                    }
                }
            }
        }

        rendered
    }

    fn bind_arguments(
        &self,
        params: &[String],
        args: &[Expression],
    ) -> Option<HashMap<String, Expression>> {
        if params.len() != args.len() {
            return None;
        }

        let mut locals = HashMap::default();
        for (param, arg) in params.iter().zip(args.iter()) {
            locals.insert(param.clone(), arg.clone());
        }
        Some(locals)
    }

    fn create_nested_runtime(
        &self,
        variables: HashMap<String, Expression>,
        functions: SharedFunctionTable,
    ) -> Runtime {
        Runtime {
            variables,
            functions,
            native_functions: self.native_functions.clone(),
            modules: self.modules.clone(),
            module_cache_by_path: self.module_cache_by_path.clone(),
        }
    }

    fn print_expression(&self, expr: &Expression) {
        match expr {
            Expression::Number(n) => println!("{}", n),
            Expression::Boolean(b) => println!("{}", b),
            Expression::StringLiteral(s) => println!("{}", s),
            Expression::StringInterpolation { parts } => {
                println!("{}", self.render_interpolation(parts))
            }
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
            Expression::StringInterpolation { parts } => {
                format!("\"{}\"", self.render_interpolation(parts))
            }
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
                let evaluated_properties: std::collections::HashMap<String, Expression> =
                    properties
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
            Expression::StringInterpolation { parts } => {
                Some(Expression::StringLiteral(self.render_interpolation(&parts)))
            }
            Expression::Variable(name) => self.variables.get(&name).cloned(),
            Expression::FunctionCall { name, args } => {
                let evaluated_args: Vec<Expression> = args
                    .into_iter()
                    .map(|arg| self.evaluate_expression(arg).unwrap_or(Expression::Null))
                    .collect();

                if name.contains('.') {
                    let parts: Vec<&str> = name.split('.').collect();
                    if parts.len() == 2 {
                        let module_name = parts[0];
                        let func_name = parts[1];
                        let module_state = {
                            let modules = self.modules.borrow();
                            modules.get(module_name).map(|module_functions| {
                                (
                                    module_functions.clone(),
                                    module_functions.get(func_name).cloned(),
                                )
                            })
                        };

                        if let Some((module_functions, Some((params, body, exported)))) =
                            module_state
                        {
                            if !exported {
                                return None;
                            }
                            if let Some(local_vars) = self.bind_arguments(&params, &evaluated_args)
                            {
                                let module_funcs_with_exported =
                                    Rc::new(RefCell::new(module_functions));
                                let mut nested_runtime = self
                                    .create_nested_runtime(local_vars, module_funcs_with_exported);
                                return nested_runtime.execute(&body);
                            }
                        }
                    }
                }

                if let Some(native_func) = self.native_functions.get(&name) {
                    native_func(evaluated_args)
                } else if let Some((params, body, _)) = self.functions.borrow().get(&name).cloned()
                {
                    if let Some(local_vars) = self.bind_arguments(&params, &evaluated_args) {
                        let mut nested_runtime =
                            self.create_nested_runtime(local_vars, self.functions.clone());
                        nested_runtime.execute(&body)
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
                    (Some(Expression::StringLiteral(l)), Some(Expression::StringLiteral(r))) => {
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
                if let Expression::Variable(module_name) = object.as_ref() {
                    if let Some(module_functions) = self.modules.borrow().get(module_name) {
                        if let Some((_, _, exported)) = module_functions.get(&property) {
                            if !exported {
                                return None;
                            }
                            return Some(Expression::Variable(format!(
                                "{}::{}",
                                module_name, property
                            )));
                        }
                    }
                }

                if let Some(evaluated_object) = self.evaluate_expression(*object) {
                    match evaluated_object {
                        Expression::Object(properties) => properties.get(&property).cloned(),
                        _ => None,
                    }
                } else {
                    None
                }
            }
        }
    }

    pub fn call_function(&mut self, name: &str, args: Vec<Expression>) -> Option<Expression> {
        self.evaluate_expression(Expression::FunctionCall {
            name: name.to_string(),
            args,
        })
    }

    pub fn get_variable(&self, name: &str) -> Option<Expression> {
        self.variables.get(name).cloned()
    }

    pub fn has_function(&self, name: &str) -> bool {
        self.functions.borrow().contains_key(name)
    }

    pub fn set_functions_snapshot(&mut self, map: FunctionTable) {
        self.functions = Rc::new(RefCell::new(map));
    }

    pub fn set_modules_snapshot(&mut self, modules: ModuleTable, module_cache: ModuleTable) {
        self.modules = Rc::new(RefCell::new(modules));
        self.module_cache_by_path = Rc::new(RefCell::new(module_cache));
    }
}
