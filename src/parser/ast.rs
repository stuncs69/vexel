// src/parser/ast.rs

#[derive(Debug, Clone)]
pub(crate) enum Statement {
    Set {
        var: String,
        value: Expression,
    },
    Function {
        name: String,
        params: Vec<String>,
        body: Vec<Statement>,
        exported: bool,
    },
    Print {
        expr: Expression,
    },
    Return {
        expr: Expression,
    },
    If {
        condition: Expression,
        body: Vec<Statement>,
    },
    FunctionCall {
        name: String,
        args: Vec<Expression>,
    },
    ForLoop {
        variable: String,
        iterable: Expression,
        body: Vec<Statement>,
    },
    WhileLoop {
        condition: Expression,
        body: Vec<Statement>,
    },
    PropertySet {
        object: Expression,
        property: String,
        value: Expression,
    },
    Import {
        module_name: String,
        file_path: String,
    },
    Test {
        name: String,
        body: Vec<Statement>,
    },
}

#[derive(Debug, Clone)]
pub(crate) enum Expression {
    Number(i32),
    Boolean(bool),
    StringLiteral(String),
    Variable(String),
    FunctionCall {
        name: String,
        args: Vec<Expression>,
    },
    Comparison {
        left: Box<Expression>,
        operator: String,
        right: Box<Expression>,
    },
    Null,
    Array(Vec<Expression>),
    Object(Vec<(String, Expression)>),
    PropertyAccess {
        object: Box<Expression>,
        property: String,
    },
}
