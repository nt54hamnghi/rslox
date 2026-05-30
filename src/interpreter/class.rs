use std::cell::RefCell;
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
    // TODO: remove pub
    pub name: String,
    pub methods: HashMap<String, LoxFunction>,
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
    fn call(&self, interpreter: &mut Interpreter, args: &[Object]) -> Result<Object, RuntimeEvent> {
        let mut instance = Rc::new(RefCell::new(LoxInstance {
            class: Rc::clone(&self),
            fields: HashMap::new(),
        }));

        if let Some(init) = self.find_method("init").cloned() {
            let obj = init.bind(instance).call(interpreter, args)?;
            match obj {
                Object::Instance(new) => instance = new,
                _ => unreachable!(),
            };
        };

        Ok(Object::Instance(instance))
    }

    fn arity(&self) -> usize {
        self.find_method("init").map(Callable::arity).unwrap_or(0)
    }
}

impl Display for LoxClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Debug, Clone)]
pub struct LoxInstance {
    // TODO: remove pub
    pub class: Rc<LoxClass>,
    pub fields: HashMap<String, Object>,
}

pub type InstanceRef = Rc<RefCell<LoxInstance>>;

impl LoxInstance {
    pub fn get(this: InstanceRef, name: &Token) -> Result<Object, RuntimeEvent> {
        if let Some(obj) = this.borrow().fields.get(&name.lexeme).cloned() {
            return Ok(obj);
        }

        let method = this.borrow().class.find_method(&name.lexeme).cloned();
        if let Some(method) = method {
            let method = method.bind(this);
            return Ok(Object::function(method));
        }

        Err(RuntimeEvent::error(
            name.clone(),
            format!("Undefined property '{}'.", name.lexeme),
        ))
    }

    pub fn set(this: InstanceRef, name: &Token, value: Object) {
        this.borrow_mut()
            .fields
            .insert(name.lexeme.to_owned(), value);
    }
}

impl Display for LoxInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} instance", self.class)
    }
}
