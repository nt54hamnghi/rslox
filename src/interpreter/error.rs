use crate::scanner::token::Token;

#[derive(Debug, thiserror::Error)]
#[error("{message}\n[line {}]", token.line)]
pub struct RuntimeError {
    token: Token,
    message: String,
}

impl RuntimeError {
    pub fn new(token: Token, message: impl Into<String>) -> Self {
        Self {
            token,
            message: message.into(),
        }
    }
}
