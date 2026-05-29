use std::fmt::{Display, Formatter, Result as FmtResult};

#[derive(Debug, Clone)]
pub struct ParseError {
    pub message: String,
    pub line: Option<usize>,
    pub column: Option<usize>,
}

impl ParseError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            line: None,
            column: None,
        }
    }

    pub fn at_line(line: usize, message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            line: Some(line),
            column: None,
        }
    }

    pub fn with_line(mut self, line: usize) -> Self {
        if self.line.is_none() {
            self.line = Some(line);
        }
        self
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match (self.line, self.column) {
            (Some(line), Some(column)) => {
                write!(f, "line {}, column {}: {}", line, column, self.message)
            }
            (Some(line), None) => write!(f, "line {}: {}", line, self.message),
            _ => write!(f, "{}", self.message),
        }
    }
}
