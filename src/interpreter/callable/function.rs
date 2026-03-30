use std::fmt::Display;
use std::rc::Rc;

use crate::interpreter::callable::Callable;
use crate::interpreter::error::RuntimeEvent;
use crate::interpreter::{Environment, EnvironmentRef, Interpreter};
use crate::parser::stmt::Function;
use crate::{Object, Value};

#[derive(Debug)]
pub(crate) struct LoxFunction {
    declaration: Function,
    closure: EnvironmentRef,
}

impl LoxFunction {
    pub fn new(declaration: Function, closure: EnvironmentRef) -> Self {
        Self {
            declaration,
            closure,
        }
    }
}

impl Callable for LoxFunction {
    fn call(&self, interpreter: &mut Interpreter, args: &[Object]) -> Result<Object, RuntimeEvent> {
        let mut env = Environment::with_enclosing(self.closure.clone());

        for (param, arg) in self.declaration.parameters.iter().zip(args) {
            env.define(param.lexeme.clone(), arg.clone());
        }

        let Err(event) = interpreter.execute_block_with(&self.declaration.body, env) else {
            return Ok(Value::Nil.into());
        };

        match event {
            RuntimeEvent::Return(obj) => Ok(obj),
            RuntimeEvent::Error(err) => Err(err.into()),
        }
    }

    fn arity(&self) -> usize {
        self.declaration.parameters.len()
    }
}

impl Display for LoxFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<fn {}>", self.declaration.name.lexeme)
    }
}

impl From<LoxFunction> for Object {
    fn from(value: LoxFunction) -> Self {
        Object::Function(Rc::new(value))
    }
}
