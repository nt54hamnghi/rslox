use std::fmt::Display;
use std::rc::Rc;

use crate::interpreter::callable::Callable;
use crate::interpreter::{Environment, Interpreter};
use crate::parser::stmt::Function;
use crate::{Object, Value};

#[derive(Debug)]
pub struct LoxFunction {
    declaration: Function,
}

impl LoxFunction {
    pub fn new(declaration: Function) -> Self {
        Self { declaration }
    }
}

impl Callable for LoxFunction {
    fn call(&self, interpreter: &mut Interpreter, args: &[Object]) -> Object {
        let mut env = Environment::with_enclosing(interpreter.environment.clone());
        for (param, arg) in self.declaration.parameters.iter().zip(args) {
            env.define(param.lexeme.clone(), arg.clone());
        }
        let res = interpreter
            .execute_block_with(&self.declaration.body, env)
            .unwrap();
        Value::Nil.into()
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
