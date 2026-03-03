use crate::parser::expr::ExprNode;

pub trait Stmt {
    fn accept<V: Visitor>(&self, visitor: &V) -> V::Output;
}

pub trait Visitor {
    type Output;
    fn visit_print_stmt(&self, stmt: &Print) -> Self::Output;
    fn visit_expression_stmt(&self, stmt: &Expression) -> Self::Output;
}

#[derive(Debug)]
pub enum StmtNode {
    Print(Print),
    Expression(Expression),
}

impl Stmt for StmtNode {
    fn accept<V: Visitor>(&self, visitor: &V) -> V::Output {
        match self {
            StmtNode::Print(stmt) => stmt.accept(visitor),
            StmtNode::Expression(stmt) => stmt.accept(visitor),
        }
    }
}

#[derive(Debug)]
pub struct Print {
    pub expr: Box<ExprNode>,
}

impl Stmt for Print {
    fn accept<V: Visitor>(&self, visitor: &V) -> V::Output {
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
pub struct Expression {
    pub expr: Box<ExprNode>,
}

impl Stmt for Expression {
    fn accept<V: Visitor>(&self, visitor: &V) -> V::Output {
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
