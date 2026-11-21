use color_eyre::eyre::bail;

use crate::object::ObjectId;

#[derive(Debug, Clone, Copy)]
#[cfg_attr(test, derive(PartialEq))]
pub enum Value {
    Nil,
    Boolean(bool),
    Number(f64),
    String(Str),
}

#[derive(Debug, Clone, Copy)]
#[cfg_attr(test, derive(PartialEq))]
pub enum Str {
    Literal(&'static str),
    Dynamic(ObjectId),
}

impl Value {
    pub fn is_falsey(&self) -> bool {
        matches!(self, Value::Nil | Value::Boolean(false))
    }
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

impl From<&'static str> for Value {
    fn from(value: &'static str) -> Self {
        Self::String(Str::Literal(value))
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

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::Value;

    #[rstest]
    #[case::nil(Value::Nil)]
    #[case::false_boolean(Value::Boolean(false))]
    fn is_falsey_for_nil_and_false(#[case] value: Value) {
        assert!(value.is_falsey());
    }

    #[rstest]
    #[case::true_boolean(Value::Boolean(true))]
    #[case::zero(Value::Number(0.0))]
    #[case::negative(Value::Number(-1.0))]
    #[case::positive(Value::Number(1.0))]
    fn is_not_falsey_for_true_and_numbers(#[case] value: Value) {
        assert!(!value.is_falsey());
    }
}
