use std::fmt::{Debug, Display};

use crate::Object;
use crate::interpreter::Interpreter;

pub mod native;

/// Represents a runtime value that can be invoked like a function.
pub trait Callable: Debug + Display {
    /// Invokes the callable with the provided interpreter state and evaluated arguments.
    fn call(&self, interpreter: &mut Interpreter, args: &[Object]) -> Object;

    /// Returns the number of arguments the callable expects.
    fn arity(&self) -> usize;
}
