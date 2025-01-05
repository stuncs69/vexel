use crate::parser::ast;

pub mod math;
pub mod debug;

pub fn get_all_native_functions() -> Vec<(&'static str, fn(Vec<ast::Expression>) -> Option<ast::Expression>)> {
    let mut functions = Vec::new();
    functions.extend(math::math_functions());
    functions.extend(debug::debug_functions());
    functions
}