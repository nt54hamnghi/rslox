use std::fmt::{Debug, Display};

use dyn_clone::DynClone;

use crate::Object;
use crate::interpreter::Interpreter;
use crate::interpreter::error::RuntimeEvent;

pub mod function;
pub mod native;

/// Represents a runtime value that can be invoked like a function.
pub trait Callable: Debug + Display + DynClone {
    /// Invokes the callable with the provided interpreter state and evaluated arguments.
    fn call(&self, interpreter: &mut Interpreter, args: &[Object]) -> Result<Object, RuntimeEvent>;

    /// Returns the number of arguments the callable expects.
    fn arity(&self) -> usize;
}

dyn_clone::clone_trait_object!(Callable);
