use crate::{Value, scanner::token::Token};

pub trait Expr {
    fn accept<V: Visitor>(&self, v: &V) -> V::Output;
}

pub trait Visitor {
    type Output;
    fn visit_literal_expr(&self, expr: &Literal) -> Self::Output;
    fn visit_grouping_expr(&self, expr: &Grouping) -> Self::Output;
    fn visit_unary_expr(&self, expr: &Unary) -> Self::Output;
    fn visit_binary_expr(&self, expr: &Binary) -> Self::Output;
}

#[derive(Debug)]
pub enum AstNode {
    Grouping(Grouping),
    Binary(Binary),
    Unary(Unary),
    Literal(Literal),
}

impl Expr for AstNode {
    fn accept<V: Visitor>(&self, v: &V) -> V::Output {
        match self {
            AstNode::Grouping(expr) => expr.accept(v),
            AstNode::Binary(expr) => expr.accept(v),
            AstNode::Unary(expr) => expr.accept(v),
            AstNode::Literal(expr) => expr.accept(v),
        }
    }
}

#[derive(Debug)]
pub struct Grouping {
    pub expression: Box<AstNode>,
}

impl Expr for Grouping {
    fn accept<V: Visitor>(&self, v: &V) -> V::Output {
        v.visit_grouping_expr(self)
    }
}

impl Grouping {
    pub fn new(expression: AstNode) -> Self {
        Self {
            expression: Box::new(expression),
        }
    }
}

impl From<Grouping> for AstNode {
    fn from(grouping: Grouping) -> Self {
        Self::Grouping(grouping)
    }
}

#[derive(Debug)]
pub struct Binary {
    pub left: Box<AstNode>,
    pub operator: Token,
    pub right: Box<AstNode>,
}

impl Expr for Binary {
    fn accept<V: Visitor>(&self, v: &V) -> V::Output {
        v.visit_binary_expr(self)
    }
}

impl Binary {
    pub fn new(left: AstNode, operator: Token, right: AstNode) -> Self {
        Self {
            left: Box::new(left),
            operator,
            right: Box::new(right),
        }
    }
}

impl From<Binary> for AstNode {
    fn from(binary: Binary) -> Self {
        Self::Binary(binary)
    }
}

#[derive(Debug)]
pub struct Unary {
    pub operator: Token,
    pub right: Box<AstNode>,
}

impl Expr for Unary {
    fn accept<V: Visitor>(&self, v: &V) -> V::Output {
        v.visit_unary_expr(self)
    }
}

impl Unary {
    pub fn new(operator: Token, right: AstNode) -> Self {
        Self {
            operator,
            right: Box::new(right),
        }
    }
}

impl From<Unary> for AstNode {
    fn from(unary: Unary) -> Self {
        Self::Unary(unary)
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Literal {
    pub value: Value,
}

impl Expr for Literal {
    fn accept<V: Visitor>(&self, v: &V) -> V::Output {
        v.visit_literal_expr(self)
    }
}

impl From<Literal> for AstNode {
    fn from(value: Literal) -> Self {
        Self::Literal(value)
    }
}

impl From<Value> for Literal {
    fn from(value: Value) -> Self {
        Literal { value }
    }
}
