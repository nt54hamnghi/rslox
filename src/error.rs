use std::fmt::Display;

use crate::interpreter::error::RuntimeError;
use crate::scanner::token::{Token, TokenType};

#[derive(Debug, thiserror::Error)]
/// Represents a scan/parse-time error with source line and optional token location.
pub struct StaticError {
    line: u32,
    location: Option<String>,
    message: String,
}

impl StaticError {
    /// Creates a static error tied to a specific source line without token context.
    pub fn error_at_line(line: u32, message: String) -> Self {
        Self {
            line,
            location: None,
            message,
        }
    }

    /// Creates a static error at a specific token location.
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

impl Display for StaticError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let location = self.location.as_deref().unwrap_or_default();
        write!(
            f,
            "[line {}] Error{}: {}",
            self.line, location, self.message
        )
    }
}

/// Application error report used across parsing and runtime stages.
///
/// `Report` wraps:
/// - `Runtime`: execution-time failures (`RuntimeError`)
/// - `Static`: scan/parse-time failures (`StaticError`)
#[derive(Debug, thiserror::Error)]
pub enum Report {
    #[error(transparent)]
    Runtime(#[from] RuntimeError),

    #[error(transparent)]
    Static(#[from] StaticError),
}

impl Report {
    /// Prints the error to stderr and terminates the process with a stage-specific code.
    ///
    /// Exit codes:
    /// - `70` for runtime errors
    /// - `65` for static (scan/parse) errors
    pub fn exit(&self) -> ! {
        match self {
            Report::Runtime(err) => {
                eprintln!("{err}");
                std::process::exit(70);
            }
            Report::Static(err) => {
                eprintln!("{err}");
                std::process::exit(65);
            }
        }
    }
}
