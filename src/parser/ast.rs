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
}
