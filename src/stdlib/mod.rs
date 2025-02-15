use crate::parser::ast;

pub mod array;
pub mod core;
pub mod debug;
pub mod math;
pub mod string;

pub fn get_all_native_functions() -> Vec<(
    &'static str,
    fn(Vec<ast::Expression>) -> Option<ast::Expression>,
)> {
    let mut functions = Vec::new();
    functions.extend(math::math_functions());
    functions.extend(array::array_functions());
    functions.extend(debug::debug_functions());
    functions.extend(string::string_functions());
    functions.extend(core::core_functions());
    functions
}
