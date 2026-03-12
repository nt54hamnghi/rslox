use std::collections::HashMap;

use crate::Value;
use crate::interpreter::error::RuntimeError;
use crate::scanner::token::Token;

#[derive(Debug, Clone, Default)]
pub(super) struct Environment {
    pub(super) values: HashMap<String, Value>,
    pub(super) enclosing: Option<Box<Environment>>,
}

impl Environment {
    /// Creates a new global [`Environment`] with no enclosing scope.
    pub(super) fn new() -> Self {
        Self {
            values: HashMap::new(),
            enclosing: None,
        }
    }

    /// Creates a new [`Environment`] with the given [`Environment`] as its enclosing scope.
    pub(super) fn with_enclosing(env: Box<Environment>) -> Self {
        Self {
            values: HashMap::new(),
            enclosing: Some(env),
        }
    }

    /// Defines a new variable in the environment by inserting the key-value pair.
    pub(super) fn define(&mut self, key: String, value: Value) {
        self.values.insert(key, value);
    }

    /// Retrieves the value of a variable from the environment.
    ///
    /// Returns a [`RuntimeError`] if the variable is not defined.
    pub(super) fn get(&self, token: &Token) -> Result<Value, RuntimeError> {
        let var_name = &token.lexeme;
        if let Some(value) = self.values.get(var_name).cloned() {
            return Ok(value);
        }

        if let Some(enclosing) = self.enclosing.as_deref() {
            return enclosing.get(token);
        }

        let msg = format!("Undefined variable '{}'.", var_name);
        Err(RuntimeError::new(token.clone(), msg))
    }

    pub(super) fn assign(&mut self, token: &Token, value: Value) -> Result<(), RuntimeError> {
        let var_name = &token.lexeme;

        if self.values.contains_key(var_name) {
            self.values.insert(var_name.clone(), value);
            return Ok(());
        }

        if let Some(enclosing) = self.enclosing.as_deref_mut() {
            return enclosing.assign(token, value);
        }

        let msg = format!("Undefined variable '{}'.", var_name);
        Err(RuntimeError::new(token.clone(), msg))
    }
}
