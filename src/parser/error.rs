// src/parser/error.rs

use std::fmt::{Display, Formatter, Result as FmtResult};

#[derive(Debug, Clone)]
pub struct ParseError {
    pub message: String,
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.message)
    }
}
