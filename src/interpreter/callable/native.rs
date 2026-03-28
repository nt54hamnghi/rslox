use std::fmt::Display;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::Object;
use crate::interpreter::Interpreter;
use crate::interpreter::callable::Callable;

#[derive(Debug)]
pub struct ClockNativeFunction;

impl Callable for ClockNativeFunction {
    fn call(&self, _interpreter: &mut Interpreter, _args: &[Object]) -> Object {
        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as f64;

        time.into()
    }

    fn arity(&self) -> usize {
        0
    }
}

impl Display for ClockNativeFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(display_native_fn())
    }
}

fn display_native_fn() -> &'static str {
    "<native fn>"
}
