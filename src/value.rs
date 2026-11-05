use std::fmt;

use color_eyre::eyre::bail;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Value {
    Nil,
    Boolean(bool),
    Number(f64),
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Self::Boolean(value)
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Self::Number(value)
    }
}

impl TryFrom<Value> for bool {
    type Error = color_eyre::Report;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Boolean(value) => Ok(value),
            value => bail!("expected boolean, got {value:?}"),
        }
    }
}

impl TryFrom<Value> for f64 {
    type Error = color_eyre::Report;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Number(value) => Ok(value),
            value => bail!("expected number, got {value:?}"),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Value::Nil => "nil".to_owned(),
            Value::Boolean(b) => format!("{b}"),
            Value::Number(n) => format!("{n}"),
        };
        write!(f, "{s}")
    }
}
