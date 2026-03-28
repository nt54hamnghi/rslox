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
    fn visit_function_stmt(&mut self, stmt: &Function) -> Self::Output;
    fn visit_if_stmt(&mut self, stmt: &If) -> Self::Output;
    fn visit_while_stmt(&mut self, stmt: &While) -> Self::Output;
    fn visit_block_stmt(&mut self, stmt: &Block) -> Self::Output;
}

#[derive(Debug, Clone)]
pub enum StmtNode {
    Print(Print),
    Expression(Expression),
    Var(Var),
    Function(Function),
    If(If),
    While(While),
    Block(Block),
}

impl Stmt for StmtNode {
    fn accept<V: Visitor>(&self, visitor: &mut V) -> V::Output {
        match self {
            StmtNode::Print(print) => print.accept(visitor),
            StmtNode::Expression(expression) => expression.accept(visitor),
            StmtNode::Var(var) => var.accept(visitor),
            StmtNode::Block(block) => block.accept(visitor),
            StmtNode::If(if_stmt) => if_stmt.accept(visitor),
            StmtNode::While(while_stmt) => while_stmt.accept(visitor),
            StmtNode::Function(function) => function.accept(visitor),
        }
    }
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct Function {
    pub name: Token,
    pub parameters: Vec<Token>,
    pub body: Vec<StmtNode>,
}

impl Stmt for Function {
    fn accept<V: Visitor>(&self, visitor: &mut V) -> V::Output {
        visitor.visit_function_stmt(&self)
    }
}

impl Function {
    pub fn new(name: Token, parameters: Vec<Token>, body: Vec<StmtNode>) -> Self {
        Self {
            name,
            parameters,
            body,
        }
    }
}

impl From<Function> for StmtNode {
    fn from(function: Function) -> Self {
        Self::Function(function)
    }
}

#[derive(Debug, Clone)]
pub struct Block {
    pub statements: Vec<StmtNode>,
}

impl Stmt for Block {
    fn accept<V: Visitor>(&self, visitor: &mut V) -> V::Output {
        visitor.visit_block_stmt(&self)
    }
}

#[derive(Debug, Clone)]
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
    fn from(if_stmt: If) -> Self {
        Self::If(if_stmt)
    }
}

#[derive(Debug, Clone)]
pub struct While {
    pub condition: Box<ExprNode>,
    pub body: Box<StmtNode>,
}

impl Stmt for While {
    fn accept<V: Visitor>(&self, visitor: &mut V) -> V::Output {
        visitor.visit_while_stmt(&self)
    }
}

impl While {
    pub fn new(condition: ExprNode, body: StmtNode) -> Self {
        Self {
            condition: Box::new(condition),
            body: Box::new(body),
        }
    }
}

impl From<While> for StmtNode {
    fn from(while_stmt: While) -> Self {
        Self::While(while_stmt)
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

#[derive(Debug, Clone)]
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
