use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Not;
use std::rc::Rc;

use slotmap::SecondaryMap;

use crate::interpreter::callable::function::LoxFunction;
use crate::interpreter::callable::native::ClockNativeFunction;
use crate::interpreter::class::LoxClass;
use crate::interpreter::environment::{Environment, EnvironmentRef};
use crate::interpreter::error::RuntimeEvent;
use crate::parser::ast::NodeId;
use crate::parser::expr::{self, Binary, Expr, ExprNode};
use crate::parser::stmt::{self, Stmt, StmtNode};
use crate::scanner::token::{Token, TokenType};
use crate::{Object, Value};

pub(crate) mod callable;
mod class;
mod environment;
pub mod error;

impl Object {
    /// Check whether a Lox object is truthy, which is defined as
    /// `nil` is false, booleans keep their value, and all other values are true.
    fn is_truthy(&self) -> bool {
        match self {
            Object::Function(_) => true,
            Object::Primitive(value) => match value {
                Value::Nil => false,
                Value::Boolean(b) => *b,
                _ => true,
            },
        }
    }
}

#[derive(Debug)]
pub struct Interpreter {
    /// Currently entered environment
    environment: EnvironmentRef,
    /// Global environment
    globals: EnvironmentRef,
    /// Locals map of variable usages to resolved locations
    locals: SecondaryMap<NodeId, usize>,
}

fn new_globals() -> Rc<RefCell<Environment>> {
    let globals = Environment::from(HashMap::from([(
        "clock".to_owned(),
        Object::function(ClockNativeFunction),
    )]));

    Rc::new(RefCell::new(globals))
}

impl Interpreter {
    pub fn new(locals: SecondaryMap<NodeId, usize>) -> Self {
        let globals = new_globals();

        Self {
            // the interpreter starts with the global environment
            // as its current environment
            environment: Rc::clone(&globals),
            globals,
            locals,
        }
    }

    pub fn interpret(&mut self, program: &[StmtNode]) -> Result<(), RuntimeEvent> {
        for statement in program {
            self.execute(statement)?;
        }
        Ok(())
    }

    /// Executes a single statement node.
    ///
    /// Returns a [`RuntimeEvent`] if execution of the statement fails at runtime.
    pub fn execute(&mut self, stmt: &StmtNode) -> Result<(), RuntimeEvent> {
        Stmt::accept(stmt, self)
    }

    /// Evaluates a single expression tree.
    ///
    /// Returns the resulting value or a runtime error when evaluation fails.
    pub fn evaluate(&mut self, expr: &ExprNode) -> Result<Object, RuntimeEvent> {
        Expr::accept(expr, self)
    }

    fn lookup_variable(&self, name: &Token, expr: &impl Expr) -> Result<Object, RuntimeEvent> {
        // the locals map only contains resolutions for local variables
        // fallback to the global environment if a distance is not found
        match self.locals.get(expr.id()) {
            Some(distance) => {
                // unwrap is safe here as the resolved distance exists for the variable in the locals map,
                // which is from the bindings provided by the resolver walking the AST and resolving variables.
                // It's a bug if the resolver fails to do so correctly.
                let value = self.environment.borrow().get_at(name, *distance).unwrap();
                Ok(value)
            }
            None => self.globals.borrow().get(name),
        }
    }
}

impl stmt::Visitor for Interpreter {
    type Output = Result<(), RuntimeEvent>;

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
            .map(|e| self.evaluate(&e))
            .transpose()?
            .unwrap_or(Object::Primitive(Value::Nil));

        self.environment.borrow_mut().define(&stmt.name, value);

        Ok(())
    }

    fn visit_function_stmt(&mut self, stmt: &stmt::Function) -> Self::Output {
        let function = LoxFunction::new(stmt.clone(), self.environment.clone());
        self.environment
            .borrow_mut()
            .define(&stmt.name, function.into());
        Ok(())
    }

    fn visit_return_stmt(&mut self, stmt: &stmt::Return) -> Self::Output {
        let value = stmt
            .value
            .as_ref()
            .map(|expr| self.evaluate(&expr))
            .transpose()?
            .unwrap_or(Value::Nil.into());

        Err(RuntimeEvent::Return(value))
    }

    fn visit_if_stmt(&mut self, stmt: &stmt::If) -> Self::Output {
        if self.evaluate(&stmt.condition)?.is_truthy() {
            return self.execute(&stmt.then_branch);
        } else if let Some(else_branch) = stmt.else_branch.as_ref() {
            return self.execute(else_branch);
        };

        Ok(())
    }

    fn visit_while_stmt(&mut self, stmt: &stmt::While) -> Self::Output {
        loop {
            let cond = self.evaluate(&stmt.condition)?;
            if !cond.is_truthy() {
                break Ok(());
            }
            self.execute(&stmt.body)?;
        }
    }

    fn visit_block_stmt(&mut self, stmt: &stmt::Block) -> Self::Output {
        self.execute_block(&stmt.statements)
    }

    fn visit_class_stmt(&mut self, stmt: &stmt::Class) -> Self::Output {
        self.environment
            .borrow_mut()
            .define(&stmt.name, Object::nil());

        let class = LoxClass::new(stmt.name.lexeme.to_owned());

        self.environment
            .borrow_mut()
            .assign(&stmt.name, Object::function(class))?;

        Ok(())
    }
}

/// A guard that restores the interpreter's previous environment
/// when temporary block execution ends.
struct BlockGuard<'i> {
    interpreter: &'i mut Interpreter,
    previous: EnvironmentRef,
}

impl<'i> Drop for BlockGuard<'i> {
    fn drop(&mut self) {
        self.interpreter.environment = self.previous.clone();
    }
}

impl Interpreter {
    /// Creates a fresh environment for a new lexical block scope.
    ///
    /// This is used for ordinary `{ ... }` blocks. The new environment is
    /// enclosed by the current one so name lookup follows the surrounding scope
    /// chain. The returned [`BlockGuard`] owns the previous environment and
    /// restores it when dropped.
    fn enter_block<'i>(&'i mut self) -> BlockGuard<'i> {
        let current = self.environment.clone();
        let new_enclosed = Environment::with_enclosing(current.clone());
        self.environment = Rc::new(RefCell::new(new_enclosed));

        BlockGuard {
            interpreter: self,
            previous: current.clone(),
        }
    }

    /// Executes statements in a new lexical block scope.
    ///
    /// Each statement runs with a new environment whose enclosing parent is the
    /// environment active at the call site. The previous environment is restored
    /// automatically, even if execution returns early with an error.
    fn execute_block(&mut self, stmts: &[StmtNode]) -> Result<(), RuntimeEvent> {
        let guard = self.enter_block();
        for stmt in stmts {
            if let Err(err) = guard.interpreter.execute(stmt) {
                return Err(err);
            }
        }
        Ok(())
    }

    /// Installs a caller-provided environment for block execution.
    ///
    /// This is used when the caller has already prepared the environment, such as
    /// a function call frame with bound parameters. The returned [`BlockGuard`]
    /// restores the previous interpreter environment on drop.
    fn enter_block_with<'i>(&'i mut self, env: Environment) -> BlockGuard<'i> {
        let current = self.environment.clone();
        self.environment = Rc::new(RefCell::new(env));

        BlockGuard {
            interpreter: self,
            previous: current.clone(),
        }
    }

    /// Executes statements using an environment supplied by the caller.
    ///
    /// Unlike [`Interpreter::execute_block`], this does not allocate a child
    /// environment on its own. It swaps in `env`, executes the statements, and
    /// restores the previous environment afterward.
    fn execute_block_with(
        &mut self,
        stmts: &[StmtNode],
        env: Environment,
    ) -> Result<(), RuntimeEvent> {
        let guard = self.enter_block_with(env);
        for stmt in stmts {
            if let Err(err) = guard.interpreter.execute(stmt) {
                return Err(err);
            }
        }
        Ok(())
    }
}

impl expr::Visitor for Interpreter {
    type Output = Result<Object, RuntimeEvent>;

    /// Produces the value represented by a literal expression.
    fn visit_literal_expr(&mut self, expr: &expr::Literal) -> Self::Output {
        Ok(expr.value.clone().into())
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
                let Object::Primitive(Value::Number(n)) = right else {
                    return Err(RuntimeEvent::error(
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

    fn visit_variable_expr(&mut self, expr: &expr::Variable) -> Self::Output {
        self.lookup_variable(&expr.name, expr)
    }

    fn visit_assign_expr(&mut self, expr: &expr::Assign) -> Self::Output {
        let value = self.evaluate(&expr.value)?;

        match self.locals.get(expr.id()) {
            Some(distance) => {
                self.environment
                    .borrow_mut()
                    .assign_at(&expr.name, value.clone(), *distance);
            }
            None => self
                .environment
                .borrow_mut()
                .assign(&expr.name, value.clone())?,
        };

        Ok(value)
    }

    /// Evaluates binary operators including arithmetic, comparison, and equality.
    ///
    /// Returns an error for invalid operand types or invalid numeric operations.
    fn visit_binary_expr(&mut self, expr: &Binary) -> Self::Output {
        let Object::Primitive(left) = self.evaluate(&expr.left)? else {
            return Err(RuntimeEvent::error(
                expr.operator.clone(),
                "Left operand must be a primitive value.",
            ));
        };
        let Object::Primitive(right) = self.evaluate(&expr.right)? else {
            return Err(RuntimeEvent::error(
                expr.operator.clone(),
                "Right operand must be a primitive value.",
            ));
        };

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
                    return Err(RuntimeEvent::error(expr.operator.clone(), "Division by 0"));
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
                _ => Err(RuntimeEvent::error(
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

    fn visit_logical_expr(&mut self, expr: &expr::Logical) -> Self::Output {
        let left = self.evaluate(&expr.left)?;

        match expr.operator.typ {
            TokenType::Or if left.is_truthy() => Ok(left),
            TokenType::And if !left.is_truthy() => Ok(left),
            _ => self.evaluate(&expr.right),
        }
    }

    fn visit_call_expr(&mut self, expr: &expr::Call) -> Self::Output {
        let Object::Function(callee) = self.evaluate(&expr.callee)? else {
            return Err(RuntimeEvent::error(
                expr.paren.clone(),
                "Can only call functions and classes.",
            ));
        };

        let args = expr
            .arguments
            .iter()
            .map(|arg| self.evaluate(arg))
            .collect::<Result<Vec<_>, _>>()?;

        if args.len() != callee.arity() {
            return Err(RuntimeEvent::error(
                expr.paren.clone(),
                format!(
                    "Expected {} arguments but got {}.",
                    callee.arity(),
                    args.len()
                ),
            ));
        }

        callee.call(self, args.as_slice())
    }
}

/// Converts two runtime values into numeric operands for arithmetic/comparison.
///
/// Returns a [`RuntimeError`] if either operand is not a number.
fn check_number_operands(left: Value, right: Value, op: Token) -> Result<(f64, f64), RuntimeEvent> {
    let (Value::Number(a), Value::Number(b)) = (left, right) else {
        return Err(RuntimeEvent::error(op, "Operands must be numbers."));
    };
    Ok((a, b))
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::error::Report;
    use crate::parser::Parser;
    use crate::resolver::Resolver;
    use crate::scanner::{ScanItem, Scanner};

    fn assert_primitive_output(expected: Value, actual: Object) {
        let Object::Primitive(actual) = actual else {
            panic!("expected primitive output, got function object");
        };
        assert_eq!(expected, actual);
    }

    fn eval_expr(input: &str) -> Result<Object, RuntimeEvent> {
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

        let mut interpreter = Interpreter::new(SecondaryMap::new());
        interpreter.evaluate(&expr)
    }

    fn interpret_program(input: &str) -> Result<(), Report> {
        let tokens = Scanner::new(input)
            .scan_tokens()
            .filter_map(|r| match r {
                Ok(ScanItem::Token(tkn)) => Some(tkn),
                Ok(ScanItem::Ignore) => None,
                Err(_) => None,
            })
            .collect::<Vec<_>>();

        let mut parser = Parser::from(tokens);
        let ast = parser.parse().expect("Expected a valid program");

        let resolver = Resolver::new();
        let bindings = resolver.resolve(&ast.stmts)?;

        let mut interpreter = Interpreter::new(bindings);
        interpreter.interpret(&ast.stmts)?;

        Ok(())
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
        assert_primitive_output(expected_output, output);
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
        assert_primitive_output(expected_output, output);
    }

    #[rstest]
    #[case("(true)", Value::Boolean(true))]
    #[case("(36)", Value::Number(36.0))]
    #[case(r#"("foo baz")"#, Value::String("foo baz".to_string()))]
    #[case("((false))", Value::Boolean(false))]
    fn test_interpreter_grouping_expressions(#[case] input: &str, #[case] expected_output: Value) {
        let output = eval_expr(input).expect("Expected evaluation to succeed");
        assert_primitive_output(expected_output, output);
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
        assert_primitive_output(expected_output, output);
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
        assert_primitive_output(expected_output, output);
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
        assert_primitive_output(expected_output, output);
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
        assert_primitive_output(expected_output, output);
    }

    #[rstest]
    #[case(r#""bar" != "world""#, Value::Boolean(true))]
    #[case(r#""bar" == "bar""#, Value::Boolean(true))]
    #[case(r#"92 == "92""#, Value::Boolean(false))]
    #[case("79 == (36 + 43)", Value::Boolean(true))]
    fn test_interpreter_equality_operators(#[case] input: &str, #[case] expected_output: Value) {
        let output = eval_expr(input).expect("Expected evaluation to succeed");
        assert_primitive_output(expected_output, output);
    }

    #[rstest]
    #[case("18 > -44", Value::Boolean(true))]
    #[case("18 <= 118", Value::Boolean(true))]
    #[case("74 >= 74", Value::Boolean(true))]
    #[case("(29 - 55) >= -(36 / 18 + 30)", Value::Boolean(true))]
    fn test_interpreter_relational_operators(#[case] input: &str, #[case] expected_output: Value) {
        let output = eval_expr(input).expect("Expected evaluation to succeed");
        assert_primitive_output(expected_output, output);
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
            // As hello is not declared before.
            var baz = hello;
        "#,
        "Undefined variable 'hello'.\n[line 3]"
    )]
    fn test_variable_runtime_errors_without_stdout(
        #[case] program: &str,
        #[case] expected_error: &str,
    ) {
        let err = interpret_program(program).expect_err("expected runtime error");
        assert_eq!(expected_error, err.to_string());
    }
}
