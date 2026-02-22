use std::ops::Not;

use crate::Value;
use crate::interpreter::error::RuntimeError;
use crate::parser::expr::{AstNode, Binary, Expr, Grouping, Literal, Unary, Visitor};
use crate::scanner::token::{Token, TokenType};

/// Error types returned when expression evaluation fails at runtime.
pub mod error;

impl Value {
    /// Check whether a Lox value is truthy, which is defined as
    /// `nil` is false, booleans keep their value, and all other values are true.
    fn is_truthy(&self) -> bool {
        match self {
            Value::Nil => false,
            Value::Boolean(b) => *b,
            _ => true,
        }
    }
}

/// Converts two runtime values into numeric operands for arithmetic/comparison.
///
/// Returns a [`RuntimeError`] if either operand is not a number.
fn check_number_operands(left: Value, right: Value, op: Token) -> Result<(f64, f64), RuntimeError> {
    let (Value::Number(a), Value::Number(b)) = (left, right) else {
        return Err(RuntimeError::new(op, "Operands must be numbers."));
    };
    Ok((a, b))
}

#[derive(Debug, Clone, Copy)]
pub struct Interpreter;

impl Interpreter {
    pub fn interpret(&self, expr: &AstNode) -> Result<(), RuntimeError> {
        let v = self.evaluate(expr)?;
        println!("{v}");
        Ok(())
    }

    /// Evaluates a single expression tree.
    ///
    /// Returns the resulting value or a runtime error when evaluation fails.
    fn evaluate(&self, expr: &AstNode) -> Result<Value, RuntimeError> {
        expr.accept(self)
    }
}

impl Visitor for Interpreter {
    type Output = Result<Value, RuntimeError>;

    /// Produces the value represented by a literal expression.
    fn visit_literal_expr(&self, expr: &Literal) -> Self::Output {
        Ok(expr.value.clone())
    }

    /// Evaluates the expression inside grouping parentheses.
    fn visit_grouping_expr(&self, expr: &Grouping) -> Self::Output {
        self.evaluate(&expr.expression)
    }

    /// Evaluates unary operators such as logical negation and numeric negation.
    ///
    /// Returns an error when numeric negation is applied to a non-number.
    fn visit_unary_expr(&self, expr: &Unary) -> Self::Output {
        let right = self.evaluate(&expr.right)?;

        match expr.operator.typ {
            TokenType::Bang => Ok(right.is_truthy().not().into()),
            TokenType::Minus => {
                let Value::Number(n) = right else {
                    return Err(RuntimeError::new(
                        expr.operator.clone(),
                        "Operand must be a number.",
                    ));
                };

                let value = -n;
                Ok(value.into())
            }
            _ => panic!(
                "Unexpected token type for unary expression, found {:?}",
                expr.operator.typ
            ),
        }
    }

    /// Evaluates binary operators including arithmetic, comparison, and equality.
    ///
    /// Returns an error for invalid operand types or invalid numeric operations.
    fn visit_binary_expr(&self, expr: &Binary) -> Self::Output {
        let left = self.evaluate(&expr.left)?;
        let right = self.evaluate(&expr.right)?;
        let op = expr.operator.clone();

        match op.typ {
            TokenType::BangEqual => Ok((left != right).into()),
            TokenType::EqualEqual => Ok((left == right).into()),
            TokenType::Minus => {
                let (a, b) = check_number_operands(left, right, op)?;
                Ok((a - b).into())
            }
            TokenType::Star => {
                let (a, b) = check_number_operands(left, right, op)?;
                Ok((a * b).into())
            }
            TokenType::Slash => {
                let (a, b) = check_number_operands(left, right, op)?;
                if b == 0f64 {
                    return Err(RuntimeError::new(expr.operator.clone(), "Division by 0"));
                }
                Ok((a / b).into())
            }
            TokenType::Greater => {
                let (a, b) = check_number_operands(left, right, op)?;
                Ok((a > b).into())
            }
            TokenType::GreaterEqual => {
                let (a, b) = check_number_operands(left, right, op)?;
                Ok((a >= b).into())
            }
            TokenType::Less => {
                let (a, b) = check_number_operands(left, right, op)?;
                Ok((a < b).into())
            }
            TokenType::LessEqual => {
                let (a, b) = check_number_operands(left, right, op)?;
                Ok((a <= b).into())
            }
            TokenType::Plus => match (left, right) {
                (Value::Number(a), Value::Number(b)) => Ok((a + b).into()),
                (Value::String(a), Value::String(b)) => Ok(format!("{a}{b}").into()),
                _ => todo!(),
            },
            _ => panic!(
                "Unexpected token type for binary expression, found {:?}",
                expr.operator.typ
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;
    use crate::scanner::{ScanItem, Scanner};
    use rstest::rstest;

    fn eval_expr(input: &str) -> Result<Value, RuntimeError> {
        let tokens = Scanner::new(input)
            .scan_tokens()
            .filter_map(|r| match r {
                Ok(ScanItem::Token(tkn)) => Some(tkn),
                Ok(ScanItem::Ignore) => None,
                Err(_) => None,
            })
            .collect::<Vec<_>>();

        let mut parser = Parser::from(tokens);
        let expr = parser.parse().expect("Expected a valid expression");
        Interpreter.evaluate(&expr)
    }

    #[rstest]
    #[case("true", Value::Boolean(true))]
    #[case("false", Value::Boolean(false))]
    #[case("nil", Value::Nil)]
    fn test_interpreter_literals_boolean_and_nil(
        #[case] input: &str,
        #[case] expected_output: Value,
    ) {
        let output = eval_expr(input).expect("Expected evaluation to succeed");
        assert_eq!(expected_output, output);
    }

    #[rstest]
    #[case("56", Value::Number(56.0))]
    #[case("87.92", Value::Number(87.92))]
    #[case(r#""foo baz""#, Value::String("foo baz".to_string()))]
    #[case(r#""88""#, Value::String("88".to_string()))]
    fn test_interpreter_literals_string_and_number(
        #[case] input: &str,
        #[case] expected_output: Value,
    ) {
        let output = eval_expr(input).expect("Expected evaluation to succeed");
        assert_eq!(expected_output, output);
    }

    #[rstest]
    #[case("(true)", Value::Boolean(true))]
    #[case("(36)", Value::Number(36.0))]
    #[case(r#"("foo baz")"#, Value::String("foo baz".to_string()))]
    #[case("((false))", Value::Boolean(false))]
    fn test_interpreter_grouping_expressions(#[case] input: &str, #[case] expected_output: Value) {
        let output = eval_expr(input).expect("Expected evaluation to succeed");
        assert_eq!(expected_output, output);
    }

    #[rstest]
    #[case("-79", Value::Number(-79.0))]
    #[case("!true", Value::Boolean(false))]
    #[case("!nil", Value::Boolean(true))]
    #[case("(!!57)", Value::Boolean(true))]
    fn test_interpreter_unary_negation_and_not(
        #[case] input: &str,
        #[case] expected_output: Value,
    ) {
        let output = eval_expr(input).expect("Expected evaluation to succeed");
        assert_eq!(expected_output, output);
    }

    #[rstest]
    #[case("17 * 34", Value::Number(578.0))]
    #[case("62 / 5", Value::Number(12.4))]
    #[case("7 * 4 / 7 / 1", Value::Number(4.0))]
    #[case("(18 * 4 / (3 * 6))", Value::Number(4.0))]
    fn test_interpreter_arithmetic_operators_1(
        #[case] input: &str,
        #[case] expected_output: Value,
    ) {
        let output = eval_expr(input).expect("Expected evaluation to succeed");
        assert_eq!(expected_output, output);
    }
}
