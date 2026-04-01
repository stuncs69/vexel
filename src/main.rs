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

    let mut run_tests = false;
    let mut file_path: Option<&str> = None;
    for arg in args.iter().skip(1) {
        if arg == "--test" {
            run_tests = true;
            continue;
        }

        if file_path.is_some() {
            eprintln!("Unexpected argument '{}'", arg);
            std::process::exit(1);
        }

        file_path = Some(arg);
    }

    let Some(file_path) = file_path else {
        eprintln!("Missing .vx file path");
        std::process::exit(1);
    };

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
            let result = if run_tests {
                runtime.execute_tests(&statements)
            } else {
                runtime.execute(&statements).map(|_| ())
            };
            if let Err(e) = result {
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
