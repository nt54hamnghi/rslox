use std::cell::RefCell;
use std::fmt::Display;
use std::rc::Rc;

use crate::interpreter::callable::Callable;
use crate::interpreter::class::LoxInstance;
use crate::interpreter::error::RuntimeEvent;
use crate::interpreter::{Environment, EnvironmentRef, Interpreter};
use crate::parser::stmt::Function;
use crate::{Object, Value};

#[derive(Debug, Clone)]
pub struct LoxFunction {
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

    pub fn bind(self, instance: LoxInstance) -> Self {
        let mut env = Environment::with_enclosing(self.closure);
        env.define_name("this", Object::instance(instance));
        Self::new(self.declaration, Rc::new(RefCell::new(env)))
    }
}

impl Callable for LoxFunction {
    fn call(&self, interpreter: &mut Interpreter, args: &[Object]) -> Result<Object, RuntimeEvent> {
        let mut env = Environment::with_enclosing(self.closure.clone());

        for (param, arg) in self.declaration.parameters.iter().zip(args) {
            env.define(param, arg.clone());
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
