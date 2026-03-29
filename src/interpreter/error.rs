use std::fmt::Debug;

use crate::Object;
use crate::scanner::token::Token;

#[derive(Debug, thiserror::Error)]
pub enum RuntimeEvent {
    #[error(transparent)]
    Error(#[from] RuntimeError),
    #[error("{0:?}")]
    Return(Object),
}

impl RuntimeEvent {
    pub fn error(token: Token, message: impl Into<String>) -> Self {
        Self::Error(RuntimeError {
            token,
            message: message.into(),
        })
    }
}

#[derive(Debug, thiserror::Error)]
#[error("{message}\n[line {}]", token.line)]
pub struct RuntimeError {
    token: Token,
    message: String,
}
