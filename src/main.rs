mod parser;
mod runtime;
mod std;
use parser::parser::parse_program;

fn main() {
    let code = r#"
    set x math_sqrt(256)

    print x
    "#;

    let statements = parse_program(code);
    let mut runtime = runtime::Runtime::new();
    runtime.execute(statements);
}
