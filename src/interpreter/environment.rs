use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::Object;
use crate::interpreter::error::RuntimeEvent;
use crate::scanner::token::Token;

pub(crate) type EnvironmentRef = Rc<RefCell<Environment>>;

#[derive(Debug, Default)]
pub(crate) struct Environment {
    pub(crate) values: HashMap<String, Object>,
    pub(crate) enclosing: Option<EnvironmentRef>,
}

impl Environment {
    /// Creates a new global [`Environment`] with no enclosing scope.
    pub(crate) fn new() -> Self {
        Self {
            values: HashMap::new(),
            enclosing: None,
        }
    }

    /// Creates a new [`Environment`] with the given [`Environment`] as its enclosing scope.
    pub(crate) fn with_enclosing(env: EnvironmentRef) -> Self {
        Self {
            values: HashMap::new(),
            enclosing: Some(env),
        }
    }

    /// Defines a new variable in the environment by inserting the key-value pair.
    pub(crate) fn define(&mut self, key: String, value: Object) {
        self.values.insert(key, value);
    }

    /// Retrieves the value of a variable from the environment.
    ///
    /// Returns a [`RuntimeError`] if the variable is not defined.
    pub(crate) fn get(&self, token: &Token) -> Result<Object, RuntimeEvent> {
        let var_name = &token.lexeme;

        if let Some(value) = self.values.get(var_name).cloned() {
            return Ok(value);
        }

        if let Some(enclosing) = self.enclosing.as_deref() {
            return enclosing.borrow().get(token);
        }

        let msg = format!("Undefined variable '{}'.", var_name);
        Err(RuntimeEvent::error(token.clone(), msg))
    }

    pub(crate) fn assign(&mut self, token: &Token, value: Object) -> Result<(), RuntimeEvent> {
        let var_name = &token.lexeme;

        if self.values.contains_key(var_name) {
            self.values.insert(var_name.clone(), value);
            return Ok(());
        }

        if let Some(enclosing) = self.enclosing.as_deref() {
            return enclosing.borrow_mut().assign(token, value);
        }

        let msg = format!("Undefined variable '{}'.", var_name);
        Err(RuntimeEvent::error(token.clone(), msg))
    }
}
