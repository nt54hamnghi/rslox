use std::cell::RefCell;
use std::fmt::Display;
use std::rc::Rc;

use crate::Object;
use crate::interpreter::callable::Callable;
use crate::interpreter::class::InstanceRef;
use crate::interpreter::error::RuntimeEvent;
use crate::interpreter::{Environment, EnvironmentRef, Interpreter};
use crate::parser::stmt::Function;

#[derive(Debug, Clone)]
pub struct LoxFunction {
    declaration: Function,
    closure: EnvironmentRef,
    is_method: bool,
}

impl LoxFunction {
    pub fn new_function(declaration: Function, closure: EnvironmentRef) -> Self {
        Self {
            declaration,
            closure,
            is_method: false,
        }
    }

    pub fn new_method(declaration: Function, closure: EnvironmentRef) -> Self {
        Self {
            declaration,
            closure,
            is_method: true,
        }
    }

    pub fn is_initializer(&self) -> bool {
        self.is_method && self.declaration.name.lexeme == "init"
    }

    pub fn bind(self, instance: InstanceRef) -> Self {
        let mut env = Environment::with_enclosing(self.closure);
        env.define("this", Object::Instance(instance));
        Self::new_method(self.declaration, Rc::new(RefCell::new(env)))
    }
}

impl Callable for LoxFunction {
    fn call(&self, interpreter: &mut Interpreter, args: &[Object]) -> Result<Object, RuntimeEvent> {
        let mut env = Environment::with_enclosing(self.closure.clone());

        for (param, arg) in self.declaration.parameters.iter().zip(args) {
            env.define(&param.lexeme, arg.clone());
        }

        let env_ref = Rc::new(RefCell::new(env));
        let ret = match interpreter.execute_block_with(&self.declaration.body, env_ref) {
            Ok(_) => Object::nil(),
            Err(RuntimeEvent::Return(obj)) => obj,
            Err(RuntimeEvent::Error(err)) => return Err(err.into()),
        };

        if self.is_initializer() {
            // unwrap is safe because self is a method bound to an instance, so its closure contains "this"
            let this = self.closure.borrow().get_at("this", 0).unwrap();
            return Ok(this);
        }

        Ok(ret)
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

#[cfg(debug_assertions)]
#[allow(dead_code)]
pub fn debug_print_environment_chain(env: &EnvironmentRef) {
    let mut current = Some(Rc::clone(env));
    let mut level = 0;

    while let Some(env_ref) = current {
        let env = env_ref.borrow();
        let label = if env.enclosing.is_some() {
            "local"
        } else {
            "global"
        };

        eprintln!("environment level {level} ({label}):");
        for (name, value) in &env.values {
            eprintln!("  {name} = {value}");
        }

        current = env.enclosing.as_ref().map(Rc::clone);
        level += 1;
    }
}
