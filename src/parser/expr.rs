use std::fmt::Display;

use crate::scanner::token::{Token, Value};

pub trait Expr {
    fn accept<V: Visitor>(self, v: V) -> V::Output;
}

pub trait Visitor {
    type Output;
    fn visit_grouping_expr<E: Expr>(&self, expr: Grouping<E>) -> Self::Output;
    fn visit_binary_expr<L: Expr, R: Expr>(&self, expr: Binary<L, R>) -> Self::Output;
    fn visit_unary_expr<R: Expr>(&self, expr: Unary<R>) -> Self::Output;
    fn visit_literal_expr(&self, expr: Literal) -> Self::Output;
}

#[derive(Debug)]
pub enum Ast {
    Grouping(Grouping<Box<Ast>>),
    Binary(Binary<Box<Ast>, Box<Ast>>),
    Unary(Unary<Box<Ast>>),
    Literal(Literal),
}

impl Expr for Ast {
    fn accept<V: Visitor>(self, v: V) -> V::Output {
        match self {
            Ast::Grouping(expr) => expr.accept(v),
            Ast::Binary(expr) => expr.accept(v),
            Ast::Unary(expr) => expr.accept(v),
            Ast::Literal(expr) => expr.accept(v),
        }
    }
}

impl Expr for Box<Ast> {
    fn accept<V: Visitor>(self, v: V) -> V::Output {
        (*self).accept(v)
    }
}

impl Ast {
    pub fn binary(left: Ast, operator: Token, right: Ast) -> Self {
        let expr = Binary {
            left: Box::new(left),
            operator,
            right: Box::new(right),
        };
        Self::Binary(expr)
    }

    pub fn unary(operator: Token, right: Ast) -> Self {
        let expr = Unary {
            operator,
            right: Box::new(right),
        };
        Self::Unary(expr)
    }

    pub fn literal(expr: Literal) -> Self {
        Self::Literal(expr)
    }

    pub(crate) fn grouping(expr: Ast) -> Ast {
        let expr = Grouping {
            expression: Box::new(expr),
        };
        Self::Grouping(expr)
    }
}

#[derive(Debug)]
pub struct Grouping<E> {
    pub expression: E,
}

impl<E: Expr> Expr for Grouping<E> {
    fn accept<V: Visitor>(self, v: V) -> V::Output {
        v.visit_grouping_expr(self)
    }
}

#[derive(Debug)]
pub struct Binary<L, R> {
    pub left: L,
    pub operator: Token,
    pub right: R,
}

impl<L: Expr, R: Expr> Expr for Binary<L, R> {
    fn accept<V: Visitor>(self, v: V) -> V::Output {
        v.visit_binary_expr(self)
    }
}

#[derive(Debug)]
pub struct Unary<R> {
    pub operator: Token,
    pub right: R,
}

impl<R: Expr> Expr for Unary<R> {
    fn accept<V: Visitor>(self, v: V) -> V::Output {
        v.visit_unary_expr(self)
    }
}

#[derive(Debug)]
pub enum Literal {
    Number(f64),
    String(String),
    Boolean(bool),
    Nil,
}

impl Expr for Literal {
    fn accept<V: Visitor>(self, v: V) -> V::Output {
        v.visit_literal_expr(self)
    }
}

impl Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Literal::Number(n) => {
                if n.fract() == 0.0 {
                    write!(f, "{:.1}", n)
                } else {
                    Display::fmt(n, f)
                }
            }
            Literal::String(s) => Display::fmt(s, f),
            Literal::Boolean(b) => Display::fmt(b, f),
            Literal::Nil => write!(f, "nil"),
        }
    }
}

impl From<&str> for Literal {
    fn from(s: &str) -> Self {
        Literal::String(s.into())
    }
}

impl From<String> for Literal {
    fn from(s: String) -> Self {
        Literal::String(s)
    }
}

impl From<f64> for Literal {
    fn from(n: f64) -> Self {
        Literal::Number(n)
    }
}

impl From<bool> for Literal {
    fn from(b: bool) -> Self {
        Literal::Boolean(b)
    }
}

impl From<Value> for Literal {
    fn from(value: Value) -> Self {
        match value {
            Value::Number(n) => n.into(),
            Value::String(s) => s.into(),
        }
    }
}
