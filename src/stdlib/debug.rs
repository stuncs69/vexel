use crate::parser::ast::Expression;

pub fn debug_functions() -> Vec<(&'static str, fn(Vec<Expression>) -> Option<Expression>)> {
    vec![
        ("dump_type", |args: Vec<Expression>| {
            println!("{:?}", args);
            if args.len() == 1 {
                println!("{:?}", args[0]);
                None
            } else {
                None
            }
        }),
    ]
}