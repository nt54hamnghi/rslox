use std::collections::HashMap;
use std::fmt::Display;
use std::rc::Rc;

use crate::Object;
use crate::interpreter::Interpreter;
use crate::interpreter::callable::Callable;
use crate::interpreter::callable::function::LoxFunction;
use crate::interpreter::error::RuntimeEvent;
use crate::scanner::token::Token;

#[derive(Debug, Clone)]
pub struct LoxClass {
    name: String,
    methods: HashMap<String, LoxFunction>,
}

impl LoxClass {
    pub fn new(name: String, methods: HashMap<String, LoxFunction>) -> Self {
        Self { name, methods }
    }

    pub fn find_method(&self, name: impl AsRef<str>) -> Option<&LoxFunction> {
        self.methods.get(name.as_ref())
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
        if let Some(obj) = self.fields.get(&name.lexeme).cloned() {
            return Ok(obj);
        }

        if let Some(method) = self.class.find_method(&name.lexeme).cloned() {
            let method = method.bind(self.clone());
            return Ok(Object::function(method));
        }

        Err(RuntimeEvent::error(
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
