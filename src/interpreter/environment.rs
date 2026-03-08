use std::collections::HashMap;

use crate::{Value, interpreter::error::RuntimeError, scanner::token::Token};

#[derive(Debug, Clone)]
pub(super) struct Environment {
    values: HashMap<String, Value>,
}

impl Environment {
    pub(super) fn new() -> Self {
        Self {
            values: HashMap::new(),
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
        self.values
            .get(var_name)
            .ok_or_else(|| {
                let msg = format!("Undefined variable '{}'.", var_name);
                RuntimeError::new(token.clone(), msg)
            })
            .cloned()
    }
}
