use std::fmt::Display;
use std::rc::Rc;

use crate::{Object, interpreter::callable::Callable};

#[derive(Debug, Clone)]
pub struct LoxClass {
    name: String,
}

impl LoxClass {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

impl Callable for LoxClass {
    fn call(
        &self,
        interpreter: &mut super::Interpreter,
        args: &[crate::Object],
    ) -> Result<crate::Object, super::error::RuntimeEvent> {
        todo!()
    }

    fn arity(&self) -> usize {
        todo!()
    }
}

impl Display for LoxClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}
