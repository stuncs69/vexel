mod parser;
mod runtime;
mod stdlib;
mod webcore;
use parser::parser::try_parse_program;
use runtime::repl::repl;
use runtime::runtime::Runtime;
use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Vexel REPL c: (with extra object support)");
        repl();
        return;
    }

    if args[1] == "webcore" {
        let folder = if args.len() >= 3 { &args[2] } else { "./" };
        webcore::run(folder);
        return;
    }

    let file_path = &args[1];

    if !file_path.ends_with(".vx") {
        eprintln!("File must have '.vx' extension");
        std::process::exit(1);
    }

    let code = match fs::read_to_string(file_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading file '{}': {}", file_path, e);
            std::process::exit(1);
        }
    };

    match try_parse_program(&code) {
        Ok(statements) => {
            let base_dir = Path::new(file_path)
                .parent()
                .unwrap_or_else(|| Path::new("."))
                .to_path_buf();
            let mut runtime = Runtime::new_with_base_dir(base_dir);
            if let Err(e) = runtime.execute(&statements) {
                eprintln!("{}", e);
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    }
}
