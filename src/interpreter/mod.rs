use std::ops::Not;

use crate::Value;
use crate::interpreter::environment::Environment;
use crate::interpreter::error::RuntimeError;
use crate::parser::expr::{self, Binary, Expr, ExprNode};
use crate::parser::stmt::{self, Stmt, StmtNode};
use crate::scanner::token::{Token, TokenType};

mod environment;
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

#[derive(Debug, Clone)]
pub struct Interpreter {
    environment: Environment,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            environment: Environment::new(),
        }
    }

    pub fn interpret(&mut self, program: &[StmtNode]) -> Result<(), RuntimeError> {
        for statement in program {
            self.execute(statement)?;
        }
        Ok(())
    }

    /// Executes a single statement node.
    ///
    /// Returns a [`RuntimeError`] if execution of the statement fails at runtime.
    pub fn execute(&mut self, stmt: &StmtNode) -> Result<(), RuntimeError> {
        Stmt::accept(stmt, self)
    }

    /// Evaluates a single expression tree.
    ///
    /// Returns the resulting value or a runtime error when evaluation fails.
    pub fn evaluate(&mut self, expr: &ExprNode) -> Result<Value, RuntimeError> {
        Expr::accept(expr, self)
    }
}

impl stmt::Visitor for Interpreter {
    type Output = Result<(), RuntimeError>;

    fn visit_print_stmt(&mut self, stmt: &stmt::Print) -> Self::Output {
        let value = self.evaluate(&stmt.expr)?;
        println!("{value}");
        Ok(())
    }

    fn visit_expression_stmt(&mut self, stmt: &stmt::Expression) -> Self::Output {
        self.evaluate(&stmt.expr)?;
        Ok(())
    }

    fn visit_var_stmt(&mut self, stmt: &stmt::Var) -> Self::Output {
        let value = stmt
            .initializer
            .as_deref()
            .map(|e| self.evaluate(e))
            .transpose()?
            .unwrap_or(Value::Nil);

        self.environment.define(stmt.name.lexeme.clone(), value);

        Ok(())
    }
}

impl expr::Visitor for Interpreter {
    type Output = Result<Value, RuntimeError>;

    /// Produces the value represented by a literal expression.
    fn visit_literal_expr(&self, expr: &expr::Literal) -> Self::Output {
        Ok(expr.value.clone())
    }

    /// Evaluates the expression inside grouping parentheses.
    fn visit_grouping_expr(&mut self, expr: &expr::Grouping) -> Self::Output {
        self.evaluate(&expr.expression)
    }

    /// Evaluates unary operators such as logical negation and numeric negation.
    ///
    /// Returns an error when numeric negation is applied to a non-number.
    fn visit_unary_expr(&mut self, expr: &expr::Unary) -> Self::Output {
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

    fn visit_variable_expr(&self, expr: &expr::Variable) -> Self::Output {
        self.environment.get(&expr.name)
    }

    fn visit_assign_expr(&mut self, expr: &expr::Assign) -> Self::Output {
        let value = self.evaluate(&expr.value)?;
        self.environment.assign(&expr.name, value.clone())?;
        Ok(value)
    }

    /// Evaluates binary operators including arithmetic, comparison, and equality.
    ///
    /// Returns an error for invalid operand types or invalid numeric operations.
    fn visit_binary_expr(&mut self, expr: &Binary) -> Self::Output {
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
                _ => Err(RuntimeError::new(
                    expr.operator.clone(),
                    "Operands must be numbers.",
                )),
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
    use rstest::rstest;

    use super::*;
    use crate::parser::Parser;
    use crate::scanner::{ScanItem, Scanner};

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
        let expr = parser
            .parse_expression()
            .expect("Expected a valid expression");
        let mut interpreter = Interpreter::new();
        interpreter.evaluate(&expr)
    }

    fn interpret_program(input: &str) -> Result<(), RuntimeError> {
        let tokens = Scanner::new(input)
            .scan_tokens()
            .filter_map(|r| match r {
                Ok(ScanItem::Token(tkn)) => Some(tkn),
                Ok(ScanItem::Ignore) => None,
                Err(_) => None,
            })
            .collect::<Vec<_>>();

        let mut parser = Parser::from(tokens);
        let program = parser.parse().expect("Expected a valid program");
        let mut interpreter = Interpreter::new();
        interpreter.interpret(&program)
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

    #[rstest]
    #[case("44 - 55", Value::Number(-11.0))]
    #[case("54 + 32 - 57", Value::Number(29.0))]
    #[case("63 + 42 - (-(34 - 95))", Value::Number(44.0))]
    #[case("(-56 + 56) * (30 * 42) / (1 + 4)", Value::Number(0.0))]
    fn test_interpreter_arithmetic_operators_2(
        #[case] input: &str,
        #[case] expected_output: Value,
    ) {
        let output = eval_expr(input).expect("Expected evaluation to succeed");
        assert_eq!(expected_output, output);
    }

    #[rstest]
    #[case(r#""hello" + "foo""#, Value::String("hellofoo".to_string()))]
    #[case(r#""quz" + "43""#, Value::String("quz43".to_string()))]
    #[case(
        r#""hello" + "hello" + "foo""#,
        Value::String("hellohellofoo".to_string())
    )]
    #[case(
        r#"("baz" + "quz") + ("world" + "baz")"#,
        Value::String("bazquzworldbaz".to_string())
    )]
    fn test_interpreter_string_concatenation(#[case] input: &str, #[case] expected_output: Value) {
        let output = eval_expr(input).expect("Expected evaluation to succeed");
        assert_eq!(expected_output, output);
    }

    #[rstest]
    #[case(r#""bar" != "world""#, Value::Boolean(true))]
    #[case(r#""bar" == "bar""#, Value::Boolean(true))]
    #[case(r#"92 == "92""#, Value::Boolean(false))]
    #[case("79 == (36 + 43)", Value::Boolean(true))]
    fn test_interpreter_equality_operators(#[case] input: &str, #[case] expected_output: Value) {
        let output = eval_expr(input).expect("Expected evaluation to succeed");
        assert_eq!(expected_output, output);
    }

    #[rstest]
    #[case("18 > -44", Value::Boolean(true))]
    #[case("18 <= 118", Value::Boolean(true))]
    #[case("74 >= 74", Value::Boolean(true))]
    #[case("(29 - 55) >= -(36 / 18 + 30)", Value::Boolean(true))]
    fn test_interpreter_relational_operators(#[case] input: &str, #[case] expected_output: Value) {
        let output = eval_expr(input).expect("Expected evaluation to succeed");
        assert_eq!(expected_output, output);
    }

    #[rstest]
    #[case(r#"-"hello""#)]
    #[case("-true")]
    #[case("-false")]
    #[case(r#"-("baz" + "bar")"#)]
    fn test_interpreter_runtime_errors_unary_operators(#[case] input: &str) {
        let err = eval_expr(input).expect_err("Expected evaluation to fail");
        assert_eq!("Operand must be a number.\n[line 1]", err.to_string());
    }

    #[rstest]
    #[case(r#"90 * "quz""#)]
    #[case(r#""baz" / 96"#)]
    #[case("true / false")]
    #[case(r#"("foo" + "quz") * ("world" + "world")"#)]
    fn test_interpreter_runtime_errors_binary_operators_1(#[case] input: &str) {
        let err = eval_expr(input).expect_err("Expected evaluation to fail");
        assert_eq!("Operands must be numbers.\n[line 1]", err.to_string());
    }

    #[rstest]
    #[case(r#""quz" + true"#)]
    #[case(r#"11 + "hello" + 76"#)]
    #[case("82 - false")]
    #[case(r#"true - ("quz" + "baz")"#)]
    fn test_interpreter_runtime_errors_binary_operators_2(#[case] input: &str) {
        let err = eval_expr(input).expect_err("Expected evaluation to fail");
        assert_eq!("Operands must be numbers.\n[line 1]", err.to_string());
    }

    #[rstest]
    #[case(r#""hello" < false"#)]
    #[case("true <= (39 + 48)")]
    #[case(r#"29 > ("hello" + "quz")"#)]
    #[case("false >= true")]
    fn test_interpreter_runtime_errors_relational_operators(#[case] input: &str) {
        let err = eval_expr(input).expect_err("Expected evaluation to fail");
        assert_eq!("Operands must be numbers.\n[line 1]", err.to_string());
    }

    #[rstest]
    #[case(
        r#"
            // Multiple statements in a single line should work
            print "baz"; print false;
            print true;
            print "bar"; print 35;
        "#
    )]
    #[case(
        r#"
            // Leading whitespace should be ignored
            print 92;
                print 92 + 30;
                    print 92 + 30 + 74;
        "#
    )]
    fn test_multiple_statements_success(#[case] program: &str) {
        let result = interpret_program(program);
        assert!(result.is_ok(), "program should execute successfully");
    }

    #[rstest]
    #[case(
        r#"
            // This program tests that statements are executed
            // even if they don't have any side effects
            (37 + 48 - 85) > (93 - 37) * 2;
            print !true;
            "bar" + "quz" + "world" == "barquzworld";
            print !true;
        "#
    )]
    #[case(
        r#"
            // This program tests statements that don't have any side effects
            80 - 50 >= -95 * 2 / 95 + 16;
            true == true;
            ("bar" == "baz") == ("quz" != "world");
            print true;
        "#
    )]
    fn test_expression_statements_success(#[case] program: &str) {
        let result = interpret_program(program);
        assert!(result.is_ok(), "program should execute successfully");
    }

    #[rstest]
    #[case(
        r#"
            // Variables are initialized to the correct value
            var quz = 10;
            print quz;
        "#
    )]
    #[case(
        r#"
            // Declares multiple variables and prints arithmetic on them
            var baz = 41;
            var bar = 41;
            print baz + bar;
            var hello = 41;
            print baz + bar + hello;
        "#
    )]
    #[case(
        r#"
            // Assigns arithmetic expression to variable, then prints it
            var foo = (8 * (79 + 79)) / 4 + 79;
            print foo;
        "#
    )]
    #[case(
        r#"
            // Declares variables and performs operations on them
            var quz = 94;
            var foo = quz;
            print foo + quz;
        "#
    )]
    fn test_variable_declarations_success(#[case] program: &str) {
        let result = interpret_program(program);
        assert!(result.is_ok(), "program should execute successfully");
    }

    #[rstest]
    #[case(
        r#"
            // Declares a variable without initializing it, so its value is nil.
            var quz;
            print quz;
        "#
    )]
    #[case(
        r#"
            // Declares an initialized variable and an uninitialized variable.
            var quz = "bar";
            var baz;
            print baz;
        "#
    )]
    #[case(
        r#"
            // Multiple uninitialized variables should default to nil.
            var bar = 29;
            var quz;
            var world;
            print quz;
        "#
    )]
    #[case(
        r#"
            // Uninitialized variables remain nil alongside initialized ones.
            var bar = 33 + 87 * 95;
            print bar;
            var quz = 87 * 95;
            print bar + quz;
            var world;
            print world;
        "#
    )]
    fn test_variable_initialization_success(#[case] program: &str) {
        let result = interpret_program(program);
        assert!(result.is_ok(), "program should execute successfully");
    }

    #[rstest]
    #[case(
        r#"
            var world = "before";
            print world;
            var world = "after";
            print world;
        "#
    )]
    #[case(
        r#"
            var hello = "after";
            var hello = "before";
            // Using a previously declared variable's value to initialize a new variable should work.
            var hello = hello;
            print hello;
        "#
    )]
    #[case(
        r#"
            // This program declares and initializes multiple variables and prints their values.
            var bar = 2;
            print bar;
            var bar = 3;
            print bar;
            var baz = 5;
            print baz;
            var bar = baz;
            print bar;
        "#
    )]
    fn test_variable_redeclaration_success(#[case] program: &str) {
        let result = interpret_program(program);
        assert!(result.is_ok(), "program should execute successfully");
    }

    #[rstest]
    #[case(
        r#"
            // This program tests that + only supports number+number or string+string
            print "the expression below is invalid";
            39 + "world";
            print "this should not be printed";
        "#,
        "Operands must be numbers.\n[line 4]"
    )]
    #[case(
        r#"
            print "62" + "foo";
            print true * (64 + 13);
        "#,
        "Operands must be numbers.\n[line 3]"
    )]
    fn test_expression_statement_runtime_errors(
        #[case] program: &str,
        #[case] expected_error: &str,
    ) {
        let err = interpret_program(program).expect_err("expected runtime error");
        assert_eq!(expected_error, err.to_string());
    }

    #[rstest]
    #[case(
        r#"
            // This program tries to access a variable before it is declared.
            print 34;
            print x;
        "#,
        "Undefined variable 'x'.\n[line 4]"
    )]
    #[case(
        r#"
            // This program tries to access a variable before it is declared.
            var world = 56;
            print bar;
        "#,
        "Undefined variable 'bar'.\n[line 4]"
    )]
    #[case(
        r#"
            // This program tries to access a variable before it is declared.
            var hello = 73;
            var result = (hello + quz) / foo;
            print result;
        "#,
        "Undefined variable 'quz'.\n[line 4]"
    )]
    #[case(
        r#"
            // This program tries to access a variable before it is declared.
            var bar = 73;
            var world = 95;
            var hello = 54;
            print bar + world + hello + quz; print 30;
        "#,
        "Undefined variable 'quz'.\n[line 6]"
    )]
    #[case(
        r#"
            // As hello is not declared before.
            var baz = hello;
        "#,
        "Undefined variable 'hello'.\n[line 3]"
    )]
    fn test_variable_runtime_errors_undefined_variable(
        #[case] program: &str,
        #[case] expected_error: &str,
    ) {
        let err = interpret_program(program).expect_err("expected runtime error");
        assert_eq!(expected_error, err.to_string());
    }
}
