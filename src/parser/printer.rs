use crate::Value;
use crate::parser::expr::{Binary, Expr, Grouping, Literal, Unary, Visitor};
use crate::scanner::token::{Token, TokenType};

#[derive(Clone, Copy)]
pub struct AstPrinter;

impl AstPrinter {
    pub fn print<E: Expr>(self, expr: &E) -> String {
        expr.accept(&self)
    }
}

macro_rules! parenthesize {
    ($visitor:ident, $name:expr, $($expression:expr),+) => {{
        let mut output = format!("({}", $name);
        $(
            output.push(' ');
            output.push_str(&$expression.accept($visitor));
        )+
        output.push(')');
        output
    }};
}

impl Visitor for AstPrinter {
    type Output = String;

    fn visit_grouping_expr(&self, expr: &Grouping) -> Self::Output {
        let Grouping { expression } = expr;
        parenthesize!(self, "group", expression)
    }

    fn visit_binary_expr(&self, expr: &Binary) -> Self::Output {
        let Binary {
            left,
            operator,
            right,
        } = expr;
        parenthesize!(self, operator.lexeme, left, right)
    }

    fn visit_unary_expr(&self, expr: &Unary) -> Self::Output {
        let Unary { operator, right } = expr;
        parenthesize!(self, operator.lexeme, right)
    }

    fn visit_literal_expr(&self, expr: &Literal) -> Self::Output {
        format!("{:?}", expr.value)
    }
}

pub fn print_example() {
    let plus = Token::new(TokenType::Plus, "+".into(), None, 1);
    let minus = Token::new(TokenType::Minus, "-".into(), None, 1);

    let left = Literal::from(Value::from(0.0));
    let right = Grouping::new(Unary::new(minus, Literal::from(Value::from(42.0)).into()).into());
    let expr = Binary::new(left.into(), plus, right.into());

    let printer = AstPrinter;
    let s = printer.print(&expr);
    println!("{}", s);
}
