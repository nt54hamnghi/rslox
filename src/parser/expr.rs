use crate::Value;
use crate::scanner::token::Token;

pub trait Expr {
    fn accept<V: Visitor>(&self, v: &mut V) -> V::Output;
}

pub trait Visitor {
    type Output;
    fn visit_literal_expr(&self, expr: &Literal) -> Self::Output;
    fn visit_grouping_expr(&mut self, expr: &Grouping) -> Self::Output;
    fn visit_unary_expr(&mut self, expr: &Unary) -> Self::Output;
    fn visit_variable_expr(&self, expr: &Variable) -> Self::Output;
    fn visit_assign_expr(&mut self, expr: &Assign) -> Self::Output;
    fn visit_binary_expr(&mut self, expr: &Binary) -> Self::Output;
}

#[derive(Debug)]
pub enum ExprNode {
    Grouping(Grouping),
    Binary(Binary),
    Unary(Unary),
    Variable(Variable),
    Assign(Assign),
    Literal(Literal),
}

impl Expr for ExprNode {
    fn accept<V: Visitor>(&self, v: &mut V) -> V::Output {
        match self {
            ExprNode::Grouping(grouping) => grouping.accept(v),
            ExprNode::Binary(binary) => binary.accept(v),
            ExprNode::Unary(unary) => unary.accept(v),
            ExprNode::Literal(literal) => literal.accept(v),
            ExprNode::Variable(variable) => variable.accept(v),
            ExprNode::Assign(assign) => assign.accept(v),
        }
    }
}

#[derive(Debug)]
pub struct Grouping {
    pub expression: Box<ExprNode>,
}

impl Expr for Grouping {
    fn accept<V: Visitor>(&self, v: &mut V) -> V::Output {
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
    fn accept<V: Visitor>(&self, v: &mut V) -> V::Output {
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
    fn accept<V: Visitor>(&self, v: &mut V) -> V::Output {
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

#[derive(Debug)]
pub struct Variable {
    pub name: Token,
}

impl Expr for Variable {
    fn accept<V: Visitor>(&self, v: &mut V) -> V::Output {
        v.visit_variable_expr(self)
    }
}

impl Variable {
    pub fn new(name: Token) -> Self {
        Self { name }
    }
}

impl From<Variable> for ExprNode {
    fn from(variable: Variable) -> Self {
        Self::Variable(variable)
    }
}

#[derive(Debug)]
pub struct Assign {
    pub name: Token,
    pub value: Box<ExprNode>,
}

impl Expr for Assign {
    fn accept<V: Visitor>(&self, v: &mut V) -> V::Output {
        v.visit_assign_expr(self)
    }
}

impl Assign {
    pub fn new(name: Token, value: ExprNode) -> Self {
        Self {
            name,
            value: Box::new(value),
        }
    }
}

impl From<Assign> for ExprNode {
    fn from(assign: Assign) -> Self {
        Self::Assign(assign)
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Literal {
    pub value: Value,
}

impl Expr for Literal {
    fn accept<V: Visitor>(&self, v: &mut V) -> V::Output {
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
