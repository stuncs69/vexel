use crate::parser::ast::{Expression, InterpolationPart, Statement};
use crate::parser::parser::try_parse_program;
use crate::stdlib::get_all_native_functions;
use rustc_hash::FxHashMap as HashMap;
use std::cell::RefCell;
use std::error::Error;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::rc::Rc;

type FunctionSignature = (Vec<String>, Vec<Statement>, bool);
type FunctionTable = HashMap<String, FunctionSignature>;
type SharedFunctionTable = Rc<RefCell<FunctionTable>>;
type NativeFunction = fn(Vec<Expression>) -> Option<Expression>;
type NativeFunctionTable = Rc<HashMap<String, NativeFunction>>;
type ModuleTable = HashMap<String, FunctionTable>;
type SharedModuleTable = Rc<RefCell<ModuleTable>>;

#[derive(Debug, Clone)]
pub struct RuntimeError {
    message: String,
}

impl RuntimeError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for RuntimeError {}

pub struct Runtime {
    variables: HashMap<String, Expression>,
    functions: SharedFunctionTable,
    native_functions: NativeFunctionTable,
    modules: SharedModuleTable,
    module_cache_by_path: SharedModuleTable,
    base_dir: PathBuf,
}

impl Runtime {
    pub(crate) fn new() -> Self {
        let base_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        Self::new_with_base_dir(base_dir)
    }

    pub(crate) fn new_with_base_dir(base_dir: PathBuf) -> Self {
        let mut runtime = Runtime {
            variables: HashMap::default(),
            functions: Rc::new(RefCell::new(HashMap::default())),
            native_functions: Rc::new(HashMap::default()),
            modules: Rc::new(RefCell::new(HashMap::default())),
            module_cache_by_path: Rc::new(RefCell::new(HashMap::default())),
            base_dir,
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

    pub(crate) fn execute(&mut self, statements: &[Statement]) -> Result<Option<Expression>, RuntimeError> {
        for statement in statements {
            match statement {
                Statement::PropertySet {
                    object,
                    property,
                    value,
                } => {
                    let evaluated_value = self.evaluate_expression(value.clone())?;

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
                            _ => {
                                return Err(RuntimeError::new(
                                    "Property assignment requires a variable/object path",
                                ));
                            }
                        }
                    };

                    path.reverse();
                    path.push(property.clone());

                    if let Some(mut root_value) = self.variables.remove(&root_var) {
                        if !update_property_in_expr(&mut root_value, &path, evaluated_value) {
                            return Err(RuntimeError::new("Failed to set nested property"));
                        }
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
                    let iterable_value = self.evaluate_expression(iterable.clone())?;
                    let Expression::Array(elements) = iterable_value else {
                        return Err(RuntimeError::new("for loop iterable must evaluate to an array"));
                    };

                    for element in elements {
                        self.variables.insert(variable.clone(), element);
                        if let Some(return_value) = self.execute(body)? {
                            return Ok(Some(return_value));
                        }
                    }
                }
                Statement::WhileLoop { condition, body } => loop {
                    let cond_value = self.evaluate_expression(condition.clone())?;
                    match cond_value {
                        Expression::Boolean(true) => {
                            if let Some(return_value) = self.execute(body)? {
                                return Ok(Some(return_value));
                            }
                        }
                        Expression::Boolean(false) => break,
                        _ => {
                            return Err(RuntimeError::new(
                                "while condition must evaluate to a boolean",
                            ));
                        }
                    }
                },
                Statement::Set { var, value } => {
                    let evaluated_value = self.evaluate_expression(value.clone())?;
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
                }
                Statement::FunctionCall { name, args } => {
                    let value = self.evaluate_expression(Expression::FunctionCall {
                        name: name.clone(),
                        args: args.clone(),
                    })?;
                    self.print_expression(&value)?;
                }
                Statement::Print { expr } => {
                    let value = self.evaluate_expression(expr.clone())?;
                    self.print_expression(&value)?;
                }
                Statement::Return { expr } => {
                    return Ok(Some(self.evaluate_expression(expr.clone())?));
                }
                Statement::If { condition, body } => {
                    let cond_value = self.evaluate_expression(condition.clone())?;
                    match cond_value {
                        Expression::Boolean(true) => {
                            if let Some(return_value) = self.execute(body)? {
                                return Ok(Some(return_value));
                            }
                        }
                        Expression::Boolean(false) => {}
                        _ => {
                            return Err(RuntimeError::new(
                                "if condition must evaluate to a boolean",
                            ));
                        }
                    }
                }
                Statement::Import {
                    module_name,
                    file_path,
                } => {
                    let resolved_path = self.resolve_import_path(file_path)?;
                    let cache_key = resolved_path.to_string_lossy().to_string();

                    if let Some(cached) = self.module_cache_by_path.borrow().get(&cache_key).cloned() {
                        self.modules.borrow_mut().insert(module_name.clone(), cached);
                        continue;
                    }

                    let content = fs::read_to_string(&resolved_path).map_err(|e| {
                        RuntimeError::new(format!(
                            "Error loading module from '{}': {}",
                            resolved_path.display(),
                            e
                        ))
                    })?;

                    let module_statements = try_parse_program(&content)
                        .map_err(|e| RuntimeError::new(format!("{}", e)))?;

                    let module_base_dir = resolved_path
                        .parent()
                        .map(Path::to_path_buf)
                        .unwrap_or_else(|| self.base_dir.clone());

                    let mut module_runtime = Runtime::new_with_base_dir(module_base_dir);
                    module_runtime.execute(&module_statements)?;

                    let funcs_clone = module_runtime.functions.borrow().clone();
                    self.modules
                        .borrow_mut()
                        .insert(module_name.clone(), funcs_clone.clone());
                    self.module_cache_by_path
                        .borrow_mut()
                        .insert(cache_key, funcs_clone);
                }
                Statement::Test { name, body } => {
                    println!("Running test: {}", name);
                    let mut nested_runtime =
                        self.create_nested_runtime(HashMap::default(), self.functions.clone());
                    let _ = nested_runtime.execute(body)?;
                    println!("Test '{}' finished", name);
                }
            }
        }
        Ok(None)
    }

    fn resolve_import_path(&self, file_path: &str) -> Result<PathBuf, RuntimeError> {
        let candidate = Path::new(file_path);
        let resolved = if candidate.is_absolute() {
            candidate.to_path_buf()
        } else {
            self.base_dir.join(candidate)
        };

        Ok(fs::canonicalize(&resolved).unwrap_or(resolved))
    }

    fn render_interpolation(&self, parts: &[InterpolationPart]) -> Result<String, RuntimeError> {
        let mut rendered = String::new();

        for part in parts {
            match part {
                InterpolationPart::Text(text) => rendered.push_str(text),
                InterpolationPart::Expression(expr) => {
                    let evaluated = self.evaluate_expression(expr.clone())?;
                    match evaluated {
                        Expression::StringLiteral(s) => rendered.push_str(&s),
                        Expression::Number(n) => rendered.push_str(&n.to_string()),
                        Expression::Boolean(b) => rendered.push_str(&b.to_string()),
                        Expression::Null => rendered.push_str("null"),
                        _ => rendered.push_str(&self.expression_to_string(&evaluated)?),
                    }
                }
            }
        }

        Ok(rendered)
    }

    fn bind_arguments(
        &self,
        params: &[String],
        args: &[Expression],
        function_name: &str,
    ) -> Result<HashMap<String, Expression>, RuntimeError> {
        if params.len() != args.len() {
            return Err(RuntimeError::new(format!(
                "Function '{}' expected {} arguments but received {}",
                function_name,
                params.len(),
                args.len()
            )));
        }

        let mut locals = HashMap::default();
        for (param, arg) in params.iter().zip(args.iter()) {
            locals.insert(param.clone(), arg.clone());
        }
        Ok(locals)
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
            base_dir: self.base_dir.clone(),
        }
    }

    fn print_expression(&self, expr: &Expression) -> Result<(), RuntimeError> {
        match expr {
            Expression::Number(n) => println!("{}", n),
            Expression::Boolean(b) => println!("{}", b),
            Expression::StringLiteral(s) => println!("{}", s),
            Expression::StringInterpolation { parts } => {
                println!("{}", self.render_interpolation(parts)?)
            }
            Expression::Null => println!("null"),
            Expression::Array(arr) => {
                let elements: Vec<String> = arr
                    .iter()
                    .map(|e| self.expression_to_string(e))
                    .collect::<Result<Vec<_>, _>>()?;
                println!("[{}]", elements.join(", "));
            }
            Expression::Object(properties) => {
                let elements: Vec<String> = properties
                    .iter()
                    .map(|(key, value)| {
                        self.expression_to_string(value)
                            .map(|rendered| format!("{}: {}", key, rendered))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                println!("{{{}}}", elements.join(", "));
            }
            Expression::Variable(name) => {
                let val = self
                    .variables
                    .get(name)
                    .ok_or_else(|| RuntimeError::new(format!("Undefined variable '{}'", name)))?;
                self.print_expression(val)?;
            }
            Expression::FunctionCall { name, args } => {
                let val = self.evaluate_expression(Expression::FunctionCall {
                    name: name.clone(),
                    args: args.clone(),
                })?;
                self.print_expression(&val)?;
            }
            _ => println!(""),
        }
        Ok(())
    }

    fn expression_to_string(&self, expr: &Expression) -> Result<String, RuntimeError> {
        match expr {
            Expression::Number(n) => Ok(n.to_string()),
            Expression::Boolean(b) => Ok(b.to_string()),
            Expression::StringLiteral(s) => Ok(format!("\"{}\"", s)),
            Expression::StringInterpolation { parts } => {
                Ok(format!("\"{}\"", self.render_interpolation(parts)?))
            }
            Expression::Null => Ok("null".to_string()),
            Expression::Array(arr) => {
                let elements: Vec<String> = arr
                    .iter()
                    .map(|e| self.expression_to_string(e))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(format!("[{}]", elements.join(", ")))
            }
            Expression::Object(properties) => {
                let elements: Vec<String> = properties
                    .iter()
                    .map(|(key, value)| {
                        self.expression_to_string(value)
                            .map(|rendered| format!("{}: {}", key, rendered))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(format!("{{{}}}", elements.join(", ")))
            }
            Expression::Variable(name) => {
                let val = self
                    .variables
                    .get(name)
                    .ok_or_else(|| RuntimeError::new(format!("Undefined variable '{}'", name)))?;
                self.expression_to_string(val)
            }
            Expression::FunctionCall { name, args } => {
                let val = self.evaluate_expression(Expression::FunctionCall {
                    name: name.clone(),
                    args: args.clone(),
                })?;
                self.expression_to_string(&val)
            }
            _ => Ok("".to_string()),
        }
    }

    fn evaluate_expression(&self, expr: Expression) -> Result<Expression, RuntimeError> {
        match expr {
            Expression::Array(elements) => {
                let evaluated_elements: Vec<Expression> = elements
                    .into_iter()
                    .map(|e| self.evaluate_expression(e))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Expression::Array(evaluated_elements))
            }
            Expression::Object(properties) => {
                let mut evaluated_properties: std::collections::HashMap<String, Expression> =
                    std::collections::HashMap::new();
                for (key, value) in properties {
                    evaluated_properties.insert(key, self.evaluate_expression(value)?);
                }
                Ok(Expression::Object(evaluated_properties))
            }
            Expression::Number(n) => Ok(Expression::Number(n)),
            Expression::Null => Ok(Expression::Null),
            Expression::Boolean(b) => Ok(Expression::Boolean(b)),
            Expression::StringLiteral(s) => Ok(Expression::StringLiteral(s)),
            Expression::StringInterpolation { parts } => {
                Ok(Expression::StringLiteral(self.render_interpolation(&parts)?))
            }
            Expression::Variable(name) => self
                .variables
                .get(&name)
                .cloned()
                .ok_or_else(|| RuntimeError::new(format!("Undefined variable '{}'", name))),
            Expression::FunctionCall { name, args } => {
                let evaluated_args: Vec<Expression> = args
                    .into_iter()
                    .map(|arg| self.evaluate_expression(arg))
                    .collect::<Result<Vec<_>, _>>()?;

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
                                return Err(RuntimeError::new(format!(
                                    "Function '{}.{}' is not exported",
                                    module_name, func_name
                                )));
                            }

                            let local_vars =
                                self.bind_arguments(&params, &evaluated_args, &name)?;

                            let module_funcs_with_exported = Rc::new(RefCell::new(module_functions));
                            let mut nested_runtime =
                                self.create_nested_runtime(local_vars, module_funcs_with_exported);
                            let value = nested_runtime.execute(&body)?;
                            return Ok(value.unwrap_or(Expression::Null));
                        }
                    }
                }

                if let Some(native_func) = self.native_functions.get(&name) {
                    return native_func(evaluated_args).ok_or_else(|| {
                        RuntimeError::new(format!(
                            "Native function '{}' failed for provided arguments",
                            name
                        ))
                    });
                }

                if let Some((params, body, _)) = self.functions.borrow().get(&name).cloned() {
                    let local_vars = self.bind_arguments(&params, &evaluated_args, &name)?;
                    let mut nested_runtime =
                        self.create_nested_runtime(local_vars, self.functions.clone());
                    let value = nested_runtime.execute(&body)?;
                    return Ok(value.unwrap_or(Expression::Null));
                }

                Err(RuntimeError::new(format!("Unknown function '{}'", name)))
            }
            Expression::Comparison {
                left,
                operator,
                right,
            } => {
                let left_val = self.evaluate_expression(*left)?;
                let right_val = self.evaluate_expression(*right)?;

                match (left_val, right_val) {
                    (Expression::Number(l), Expression::Number(r)) => match operator.as_str() {
                        "==" => Ok(Expression::Boolean(l == r)),
                        "!=" => Ok(Expression::Boolean(l != r)),
                        ">" => Ok(Expression::Boolean(l > r)),
                        "<" => Ok(Expression::Boolean(l < r)),
                        ">=" => Ok(Expression::Boolean(l >= r)),
                        "<=" => Ok(Expression::Boolean(l <= r)),
                        _ => Err(RuntimeError::new(format!(
                            "Unsupported comparison operator '{}'",
                            operator
                        ))),
                    },
                    (Expression::Boolean(l), Expression::Boolean(r)) => match operator.as_str() {
                        "==" => Ok(Expression::Boolean(l == r)),
                        "!=" => Ok(Expression::Boolean(l != r)),
                        _ => Err(RuntimeError::new(format!(
                            "Unsupported comparison operator '{}' for booleans",
                            operator
                        ))),
                    },
                    (Expression::StringLiteral(l), Expression::StringLiteral(r)) => {
                        match operator.as_str() {
                            "==" => Ok(Expression::Boolean(l == r)),
                            "!=" => Ok(Expression::Boolean(l != r)),
                            _ => Err(RuntimeError::new(format!(
                                "Unsupported comparison operator '{}' for strings",
                                operator
                            ))),
                        }
                    }
                    _ => Err(RuntimeError::new(
                        "Comparison operands must both be numbers, booleans, or strings",
                    )),
                }
            }
            Expression::PropertyAccess { object, property } => {
                if let Expression::Variable(module_name) = object.as_ref() {
                    if let Some(module_functions) = self.modules.borrow().get(module_name) {
                        if let Some((_, _, exported)) = module_functions.get(&property) {
                            if !exported {
                                return Err(RuntimeError::new(format!(
                                    "Function '{}.{}' is not exported",
                                    module_name, property
                                )));
                            }
                            return Ok(Expression::Variable(format!(
                                "{}::{}",
                                module_name, property
                            )));
                        }
                    }
                }

                match self.evaluate_expression(*object)? {
                    Expression::Object(properties) => properties
                        .get(&property)
                        .cloned()
                        .ok_or_else(|| RuntimeError::new(format!("Property '{}' not found", property))),
                    _ => Err(RuntimeError::new(
                        "Property access target must evaluate to an object",
                    )),
                }
            }
        }
    }

    pub fn call_function(
        &mut self,
        name: &str,
        args: Vec<Expression>,
    ) -> Result<Option<Expression>, RuntimeError> {
        let value = self.evaluate_expression(Expression::FunctionCall {
            name: name.to_string(),
            args,
        })?;
        Ok(Some(value))
    }

    pub fn get_variable(&self, name: &str) -> Option<Expression> {
        self.variables.get(name).cloned()
    }

    pub fn has_function(&self, name: &str) -> bool {
        self.functions.borrow().contains_key(name)
    }

}
