use std::fmt::Display;

use crate::scanner::token::{Token, TokenType};

#[derive(Debug, thiserror::Error)]
pub struct Report {
    line: u32,
    location: Option<String>,
    message: String,
}

impl Report {
    pub fn error_at_line(line: u32, message: String) -> Self {
        Self {
            line,
            location: None,
            message,
        }
    }

    pub fn error_at_token(token: &Token, message: String) -> Self {
        let location = if token.typ == TokenType::Eof {
            " at end".into()
        } else {
            format!(" at '{}'", token.lexeme)
        };

        Self {
            line: token.line,
            location: Some(location),
            message,
        }
    }
}

impl Display for Report {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let location = self.location.as_deref().unwrap_or_default();
        write!(
            f,
            "[line {}] Error{}: {}",
            self.line, location, self.message
        )
    }
}
