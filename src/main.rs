mod parser;
mod runtime;
mod stdlib;
use parser::parser::parse_program;
use std::env;
use std::fs;

// fn main() {
//     let args: Vec<String> = env::args().collect();

//     if args.len() < 2 {
//         eprintln!("Usage: {} <file_path>", args[0]);
//         std::process::exit(1);
//     }

//     let file_path = &args[1];

//     if !file_path.ends_with(".vx") {
//         eprintln!("File must have '.vx' extension");
//         std::process::exit(1);
//     }

//     let code = match fs::read_to_string(file_path) {
//         Ok(content) => content,
//         Err(e) => {
//             eprintln!("Error reading file '{}': {}", file_path, e);
//             std::process::exit(1);
//         }
//     };

//     let statements = parse_program(&code);
//     let mut runtime = runtime::Runtime::new();
//     runtime.execute(statements);
// }

fn main() {
    let code = r#"
    set x 10

    set arr [1, 2, 3, 4, 5]

    for x in arr start
        print x
    end

    set isTrue true

    while isTrue start
        print "Hello, World!"
        set isTrue false
    end

    "#;

    let statements = parse_program(&code);
    println!("{:#?}", statements);
}
