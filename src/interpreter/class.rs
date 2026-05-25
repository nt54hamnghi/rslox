use std::fmt::Display;
use std::rc::Rc;

use crate::Object;
use crate::interpreter::Interpreter;
use crate::interpreter::callable::Callable;
use crate::interpreter::error::RuntimeEvent;

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
    fn call(&self, interpreter: &mut Interpreter, args: &[Object]) -> Result<Object, RuntimeEvent> {
        let instance = LoxInstance {
            class: Rc::clone(&self),
        };
        Ok(Object::Instance(instance))
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
}

impl Display for LoxInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} instance", self.class)
    }
}
