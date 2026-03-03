use crate::Value;
use crate::scanner::token::Token;

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
pub enum ExprNode {
    Grouping(Grouping),
    Binary(Binary),
    Unary(Unary),
    Literal(Literal),
}

impl Expr for ExprNode {
    fn accept<V: Visitor>(&self, v: &V) -> V::Output {
        match self {
            ExprNode::Grouping(expr) => expr.accept(v),
            ExprNode::Binary(expr) => expr.accept(v),
            ExprNode::Unary(expr) => expr.accept(v),
            ExprNode::Literal(expr) => expr.accept(v),
        }
    }
}

#[derive(Debug)]
pub struct Grouping {
    pub expression: Box<ExprNode>,
}

impl Expr for Grouping {
    fn accept<V: Visitor>(&self, v: &V) -> V::Output {
        v.visit_grouping_expr(self)
    }
}

impl Grouping {
    pub fn new(expression: ExprNode) -> Self {
        Self {
            expression: Box::new(expression),
        }
    }
}

impl From<Grouping> for ExprNode {
    fn from(grouping: Grouping) -> Self {
        Self::Grouping(grouping)
    }
}

#[derive(Debug)]
pub struct Binary {
    pub left: Box<ExprNode>,
    pub operator: Token,
    pub right: Box<ExprNode>,
}

impl Expr for Binary {
    fn accept<V: Visitor>(&self, v: &V) -> V::Output {
        v.visit_binary_expr(self)
    }
}

impl Binary {
    pub fn new(left: ExprNode, operator: Token, right: ExprNode) -> Self {
        Self {
            left: Box::new(left),
            operator,
            right: Box::new(right),
        }
    }
}

impl From<Binary> for ExprNode {
    fn from(binary: Binary) -> Self {
        Self::Binary(binary)
    }
}

#[derive(Debug)]
pub struct Unary {
    pub operator: Token,
    pub right: Box<ExprNode>,
}

impl Expr for Unary {
    fn accept<V: Visitor>(&self, v: &V) -> V::Output {
        v.visit_unary_expr(self)
    }
}

impl Unary {
    pub fn new(operator: Token, right: ExprNode) -> Self {
        Self {
            operator,
            right: Box::new(right),
        }
    }
}

impl From<Unary> for ExprNode {
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

impl From<Literal> for ExprNode {
    fn from(value: Literal) -> Self {
        Self::Literal(value)
    }
}

impl From<Value> for Literal {
    fn from(value: Value) -> Self {
        Literal { value }
    }
}
