use crate::parser::parser::try_parse_program;
use crate::Runtime;
use std::io::{self, Write};

pub(crate) fn repl() {
    let mut runtime = Runtime::new();
    let mut buffer = String::new();
    let mut in_block = false;

    loop {
        if in_block {
            print!("... ");
        } else {
            print!(">> ");
        }
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim_end();

        if input.is_empty() {
            continue;
        }

        if input == "exit" {
            break;
        }

        if (input.starts_with("function ")
            || input.starts_with("export function ")
            || input.starts_with("if ")
            || input.starts_with("for ")
            || input.starts_with("while ")
            || input.starts_with("test "))
            && input.trim_end().ends_with(" start")
        {
            in_block = true;
        }

        buffer.push_str(input);
        buffer.push('\n');

        if in_block && input == "end" {
            in_block = false;
        }

        if !in_block {
            match try_parse_program(&buffer) {
                Ok(statements) => {
                    if let Err(e) = runtime.execute(&statements) {
                        eprintln!("{}", e);
                    }
                }
                Err(e) => {
                    eprintln!("{}", e);
                }
            }
            buffer.clear();
        }
    }
}
