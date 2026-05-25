use std::fmt::{Debug, Display};

use crate::interpreter::callable::Callable;
use crate::interpreter::class::LoxInstance;

pub mod cli;
pub mod error;
pub mod interpreter;
pub mod parser;
pub mod resolver;
pub mod scanner;

#[derive(Debug, Clone)]
pub enum Object {
    Primitive(Value),
    Function(Box<dyn Callable>),
    Instance(LoxInstance),
}

impl Object {
    pub fn function<T: Callable + 'static>(fun: T) -> Self {
        Object::Function(Box::new(fun))
    }

    pub fn nil() -> Self {
        Object::Primitive(Value::Nil)
    }
}

impl<T> From<T> for Object
where
    T: Into<Value>,
{
    fn from(value: T) -> Self {
        Object::Primitive(value.into())
    }
}

impl Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Object::Primitive(value) => Display::fmt(value, f),
            Object::Function(fun) => Display::fmt(fun, f),
            Object::Instance(instance) => Display::fmt(instance, f),
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum Value {
    Number(f64),
    String(String),
    Boolean(bool),
    Nil,
}

impl Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Number(n) => Debug::fmt(n, f),
            Self::String(s) => Display::fmt(s, f), // use Display to exclude quotes
            Self::Boolean(b) => Debug::fmt(b, f),
            Self::Nil => write!(f, "nil"),
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Number(n) => Display::fmt(n, f),
            Self::String(s) => Display::fmt(s, f),
            Self::Boolean(b) => Display::fmt(b, f),
            Self::Nil => write!(f, "nil"),
        }
    }
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Value::String(s.into())
    }
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Value::String(s)
    }
}

impl From<f64> for Value {
    fn from(n: f64) -> Self {
        Value::Number(n)
    }
}

impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Value::Boolean(b)
    }
}
