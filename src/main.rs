mod parser;
mod runtime;
mod stdlib;
use parser::parser::parse_program;
use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <file_path>", args[0]);
        std::process::exit(1);
    }

    // Read the file
    let file_path = &args[1];
    let code = match fs::read_to_string(file_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading file '{}': {}", file_path, e);
            std::process::exit(1);
        }
    };

    let statements = parse_program(&code);
    let mut runtime = runtime::Runtime::new();
    runtime.execute(statements);
}