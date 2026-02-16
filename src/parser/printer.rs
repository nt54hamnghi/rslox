use crate::parser::expr::{Binary, Expr, Grouping, Literal, Unary, Visitor};
use crate::scanner::token::{Token, TokenType};

#[derive(Clone, Copy)]
pub struct AstPrinter;

impl AstPrinter {
    pub fn print<E: Expr>(self, expr: E) -> String {
        expr.accept(self)
    }
}

macro_rules! parenthesize {
    ($visitor:ident, $name:expr, $($expression:expr),+) => {{
        let mut output = format!("({}", $name);
        $(
            output.push(' ');
            output.push_str(&$expression.accept(*$visitor));
        )+
        output.push(')');
        output
    }};
}

impl Visitor for AstPrinter {
    type Output = String;

    fn visit_grouping_expr<E: Expr>(&self, expr: Grouping<E>) -> Self::Output {
        let Grouping { expression } = expr;
        parenthesize!(self, "group", expression)
    }

    fn visit_binary_expr<L: Expr, R: Expr>(&self, expr: Binary<L, R>) -> Self::Output {
        let Binary {
            left,
            operator,
            right,
        } = expr;
        parenthesize!(self, operator.lexeme, left, right)
    }

    fn visit_unary_expr<R: Expr>(&self, expr: Unary<R>) -> Self::Output {
        let Unary { operator, right } = expr;
        parenthesize!(self, operator.lexeme, right)
    }

    fn visit_literal_expr(&self, expr: Literal) -> Self::Output {
        expr.to_string()
    }
}

pub fn print_example() {
    let expr = Binary {
        left: Literal::from(0.0),
        operator: Token::new(TokenType::Plus, "+".into(), None, 1),
        right: Grouping {
            expression: Unary {
                operator: Token::new(TokenType::Plus, "-".into(), None, 1),
                right: Literal::from(42.),
            },
        },
    };

    let printer = AstPrinter;
    let s = printer.print(expr);
    println!("{}", s);
}
