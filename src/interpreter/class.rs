use std::collections::HashMap;
use std::fmt::Display;
use std::rc::Rc;

use crate::Object;
use crate::interpreter::Interpreter;
use crate::interpreter::callable::Callable;
use crate::interpreter::error::RuntimeEvent;
use crate::scanner::token::Token;

#[derive(Debug, Clone)]
pub struct LoxClass {
    name: String,
}

impl LoxClass {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

impl Callable for Rc<LoxClass> {
    fn call(
        &self,
        _interpreter: &mut Interpreter,
        _args: &[Object],
    ) -> Result<Object, RuntimeEvent> {
        let instance = LoxInstance {
            class: Rc::clone(&self),
            fields: HashMap::new(),
        };
        Ok(Object::instance(instance))
    }

    fn arity(&self) -> usize {
        0
    }
}

impl Display for LoxClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Debug, Clone)]
pub struct LoxInstance {
    class: Rc<LoxClass>,
    fields: HashMap<String, Object>,
}

impl LoxInstance {
    pub fn get(&self, name: &Token) -> Result<Object, RuntimeEvent> {
        self.fields
            .get(&name.lexeme)
            .cloned()
            .ok_or(RuntimeEvent::error(
                name.clone(),
                format!("Undefined property '{}'.", name.lexeme),
            ))
    }

    pub fn set(&mut self, name: &Token, value: Object) {
        self.fields.insert(name.lexeme.to_owned(), value);
    }
}

impl Display for LoxInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} instance", self.class)
    }
}
