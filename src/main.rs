mod parser;
mod runtime;
mod std;
use parser::parser::parse_program;

fn main() {
    let code = r#"
    set x 10

    function math_add2(a, b) start
        return math_add(a, b)
    end

    set y math_add2(x, 5)
    print y
    "#;

    let statements = parse_program(code);
    let mut runtime = runtime::Runtime::new();
    runtime.execute(statements);
}
