use crate::parser::expr::ExprNode;
use crate::scanner::token::Token;

pub trait Stmt {
    fn accept<V: Visitor>(&self, visitor: &mut V) -> V::Output;
}

pub trait Visitor {
    type Output;
    fn visit_print_stmt(&mut self, stmt: &Print) -> Self::Output;
    fn visit_expression_stmt(&mut self, stmt: &Expression) -> Self::Output;
    fn visit_var_stmt(&mut self, stmt: &Var) -> Self::Output;
    fn visit_if_stmt(&mut self, stmt: &If) -> Self::Output;
    fn visit_block_stmt(&mut self, stmt: &Block) -> Self::Output;
}

#[derive(Debug)]
pub enum StmtNode {
    Print(Print),
    Expression(Expression),
    Var(Var),
    If(If),
    Block(Block),
}

impl Stmt for StmtNode {
    fn accept<V: Visitor>(&self, visitor: &mut V) -> V::Output {
        match self {
            StmtNode::Print(print) => print.accept(visitor),
            StmtNode::Expression(expression) => expression.accept(visitor),
            StmtNode::Var(var) => var.accept(visitor),
            StmtNode::Block(block) => block.accept(visitor),
            StmtNode::If(ifs) => ifs.accept(visitor),
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
            initializer: initializer.map(Box::new),
        }
    }
}

impl From<Var> for StmtNode {
    fn from(var: Var) -> Self {
        Self::Var(var)
    }
}

#[derive(Debug)]
pub struct Block {
    pub statements: Vec<StmtNode>,
}

impl Stmt for Block {
    fn accept<V: Visitor>(&self, visitor: &mut V) -> V::Output {
        visitor.visit_block_stmt(&self)
    }
}

#[derive(Debug)]
pub struct If {
    pub condition: Box<ExprNode>,
    pub then_branch: Box<StmtNode>,
    pub else_branch: Option<Box<StmtNode>>,
}

impl Stmt for If {
    fn accept<V: Visitor>(&self, visitor: &mut V) -> V::Output {
        visitor.visit_if_stmt(&self)
    }
}

impl If {
    pub fn new(condition: ExprNode, then_branch: StmtNode, else_branch: Option<StmtNode>) -> Self {
        Self {
            condition: Box::new(condition),
            then_branch: Box::new(then_branch),
            else_branch: else_branch.map(Box::new),
        }
    }
}

impl From<If> for StmtNode {
    fn from(ifs: If) -> Self {
        Self::If(ifs)
    }
}

impl Block {
    pub fn new(statements: Vec<StmtNode>) -> Self {
        Self { statements }
    }
}

impl From<Block> for StmtNode {
    fn from(var: Block) -> Self {
        Self::Block(var)
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
