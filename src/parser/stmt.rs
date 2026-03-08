use crate::parser::expr::ExprNode;
use crate::scanner::token::Token;

pub trait Stmt {
    fn accept<V: Visitor>(&self, visitor: &mut V) -> V::Output;
}

pub trait Visitor {
    type Output;
    fn visit_print_stmt(&self, stmt: &Print) -> Self::Output;
    fn visit_expression_stmt(&self, stmt: &Expression) -> Self::Output;
    fn visit_var_stmt(&mut self, stmt: &Var) -> Self::Output;
}

#[derive(Debug)]
pub enum StmtNode {
    Print(Print),
    Expression(Expression),
    Var(Var),
}

impl Stmt for StmtNode {
    fn accept<V: Visitor>(&self, visitor: &mut V) -> V::Output {
        match self {
            StmtNode::Print(print) => print.accept(visitor),
            StmtNode::Expression(expression) => expression.accept(visitor),
            StmtNode::Var(var) => var.accept(visitor),
        }
    }
}

#[derive(Debug)]
pub struct Print {
    pub expr: Box<ExprNode>,
}

impl Stmt for Print {
    fn accept<V: Visitor>(&self, visitor: &mut V) -> V::Output {
        visitor.visit_print_stmt(&self)
    }
}

impl Print {
    pub fn new(expr: ExprNode) -> Self {
        Self {
            expr: Box::new(expr),
        }
    }
}

impl From<Print> for StmtNode {
    fn from(print: Print) -> Self {
        Self::Print(print)
    }
}

#[derive(Debug)]
pub struct Var {
    pub name: Token,
    pub initializer: Option<Box<ExprNode>>,
}

impl Stmt for Var {
    fn accept<V: Visitor>(&self, visitor: &mut V) -> V::Output {
        visitor.visit_var_stmt(&self)
    }
}

impl Var {
    pub fn new(name: Token, initializer: Option<ExprNode>) -> Self {
        Self {
            name,
            initializer: initializer.map(|i| Box::new(i)),
        }
    }
}

impl From<Var> for StmtNode {
    fn from(var: Var) -> Self {
        Self::Var(var)
    }
}

#[derive(Debug)]
pub struct Expression {
    pub expr: Box<ExprNode>,
}

impl Stmt for Expression {
    fn accept<V: Visitor>(&self, visitor: &mut V) -> V::Output {
        visitor.visit_expression_stmt(&self)
    }
}

impl Expression {
    pub fn new(expr: ExprNode) -> Self {
        Self {
            expr: Box::new(expr),
        }
    }
}

impl From<Expression> for StmtNode {
    fn from(expression: Expression) -> Self {
        Self::Expression(expression)
    }
}
