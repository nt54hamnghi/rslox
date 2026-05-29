use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::Object;
use crate::interpreter::error::RuntimeEvent;
use crate::scanner::token::Token;

pub type EnvironmentRef = Rc<RefCell<Environment>>;

#[derive(Debug, Default)]
pub struct Environment {
    pub values: HashMap<String, Object>,
    pub enclosing: Option<EnvironmentRef>,
}

impl From<HashMap<String, Object>> for Environment {
    fn from(value: HashMap<String, Object>) -> Self {
        Self {
            values: value,
            enclosing: None,
        }
    }
}

impl Environment {
    /// Creates a new [`Environment`] with the given [`Environment`] as its enclosing scope.
    pub fn with_enclosing(env: EnvironmentRef) -> Self {
        Self {
            values: HashMap::new(),
            enclosing: Some(env),
        }
    }

    /// Defines a new variable in the environment by inserting the key-value pair.
    pub fn define(&mut self, key: &Token, value: Object) {
        self.values.insert(key.lexeme.clone(), value);
    }

    /// Retrieves the value of a variable in this environment,
    /// or walks up the enclosing environment chain until the variable is found.
    ///
    /// Returns a [`RuntimeEvent`] error if the variable is not defined in any accessible scope.
    pub fn get(&self, token: &Token) -> Result<Object, RuntimeEvent> {
        let var_name = &token.lexeme;

        if let Some(value) = self.values.get(var_name).cloned() {
            return Ok(value);
        }

        if let Some(enclosing) = self.enclosing.as_ref() {
            return enclosing.borrow().get(token);
        }

        Err(RuntimeEvent::error(
            token.clone(),
            format!("Undefined variable '{}'.", var_name),
        ))
    }

    /// Retrieves a variable from the environment exactly `distance` scopes away.
    ///
    /// A distance of `0` uses this environment; each additional step follows one enclosing
    /// environment before reading the variable.
    ///
    /// Returns [`None`] if the target variable binding does not exist.
    ///
    /// # Panic
    ///
    /// Panics if `distance` requires walking past the outermost environment.
    pub fn get_at(&self, token: &Token, distance: usize) -> Option<Object> {
        if distance == 0 {
            return self.values.get(&token.lexeme).cloned();
        }
        self.enclosing
            .as_ref()
            .unwrap()
            .borrow()
            .get_at(token, distance - 1)
    }

    /// Assigns a variable by checking this environment, then walking up the enclosing
    /// environment chain until the existing binding is found.
    ///
    /// Returns a [`RuntimeEvent`] error if the variable is not defined in any accessible scope.
    pub fn assign(&mut self, token: &Token, value: Object) -> Result<(), RuntimeEvent> {
        let var_name = &token.lexeme;

        if self.values.contains_key(var_name) {
            self.values.insert(var_name.clone(), value);
            return Ok(());
        }

        if let Some(enclosing) = self.enclosing.as_deref() {
            return enclosing.borrow_mut().assign(token, value);
        }

        Err(RuntimeEvent::error(
            token.clone(),
            format!("Undefined variable '{}'.", var_name),
        ))
    }

    /// Assigns a variable in the environment exactly `distance` scopes away.
    ///
    /// A distance of `0` uses this environment; each additional step follows one enclosing
    /// environment before assigning the variable.
    ///
    /// # Panics
    ///
    /// Panics if `distance` requires walking past the outermost environment.
    pub fn assign_at(&mut self, token: &Token, value: Object, distance: usize) {
        if distance == 0 {
            self.values.insert(token.lexeme.clone(), value);
            return;
        }

        self.enclosing
            .as_ref()
            .unwrap()
            .borrow_mut()
            .assign_at(token, value, distance - 1)
    }
}
