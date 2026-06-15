use crate::parser::ast::{Context, NodeId};
use crate::parser::expr::{ExprNode, Variable};
use crate::scanner::token::Token;

pub trait Stmt {
    fn accept<V: Visitor>(&self, visitor: &mut V) -> V::Output;
    fn id(&self) -> NodeId;
}

pub trait Visitor {
    type Output;
    fn visit_print_stmt(&mut self, stmt: &Print) -> Self::Output;
    fn visit_expression_stmt(&mut self, stmt: &Expression) -> Self::Output;
    fn visit_var_stmt(&mut self, stmt: &Var) -> Self::Output;
    fn visit_function_stmt(&mut self, stmt: &Function) -> Self::Output;
    fn visit_return_stmt(&mut self, stmt: &Return) -> Self::Output;
    fn visit_if_stmt(&mut self, stmt: &If) -> Self::Output;
    fn visit_while_stmt(&mut self, stmt: &While) -> Self::Output;
    fn visit_block_stmt(&mut self, stmt: &Block) -> Self::Output;
    fn visit_class_stmt(&mut self, stmt: &Class) -> Self::Output;
}

impl Context {
    pub(super) fn new_print(&'static self, expr: ExprNode) -> StmtNode {
        let id = self
            .nodes
            .borrow_mut()
            .insert_with_key(|id| Box::new(Print { id, expr }));
        StmtNode {
            ctx: self,
            id,
            kind: StmtKind::Print,
        }
    }

    pub(super) fn new_expression(&'static self, expr: ExprNode) -> StmtNode {
        let id = self
            .nodes
            .borrow_mut()
            .insert_with_key(|id| Box::new(Expression { id, expr }));
        StmtNode {
            ctx: self,
            id,
            kind: StmtKind::Expression,
        }
    }

    pub(super) fn new_var(&'static self, name: Token, initializer: Option<ExprNode>) -> StmtNode {
        let id = self.nodes.borrow_mut().insert_with_key(|id| {
            Box::new(Var {
                id,
                name,
                initializer,
            })
        });
        StmtNode {
            ctx: self,
            id,
            kind: StmtKind::Var,
        }
    }

    pub(super) fn new_function(
        &'static self,
        name: Token,
        parameters: Vec<Token>,
        body: Vec<StmtNode>,
    ) -> StmtNode {
        let id = self.nodes.borrow_mut().insert_with_key(|id| {
            Box::new(Function {
                id,
                name,
                parameters,
                body,
            })
        });
        StmtNode {
            ctx: self,
            id,
            kind: StmtKind::Function,
        }
    }

    pub(super) fn new_return(&'static self, keyword: Token, value: Option<ExprNode>) -> StmtNode {
        let id = self
            .nodes
            .borrow_mut()
            .insert_with_key(|id| Box::new(Return { id, keyword, value }));
        StmtNode {
            ctx: self,
            id,
            kind: StmtKind::Return,
        }
    }

    pub(super) fn new_if(
        &'static self,
        condition: ExprNode,
        then_branch: StmtNode,
        else_branch: Option<StmtNode>,
    ) -> StmtNode {
        let id = self.nodes.borrow_mut().insert_with_key(|id| {
            Box::new(If {
                id,
                condition,
                then_branch,
                else_branch,
            })
        });
        StmtNode {
            ctx: self,
            id,
            kind: StmtKind::If,
        }
    }

    pub(super) fn new_while(&'static self, condition: ExprNode, body: StmtNode) -> StmtNode {
        let id = self.nodes.borrow_mut().insert_with_key(|id| {
            Box::new(While {
                id,
                condition,
                body,
            })
        });
        StmtNode {
            ctx: self,
            id,
            kind: StmtKind::While,
        }
    }

    pub(super) fn new_block(&'static self, statements: Vec<StmtNode>) -> StmtNode {
        let id = self
            .nodes
            .borrow_mut()
            .insert_with_key(|id| Box::new(Block { id, statements }));
        StmtNode {
            ctx: self,
            id,
            kind: StmtKind::Block,
        }
    }

    pub(super) fn new_class(
        &'static self,
        name: Token,
        superclass: Option<Variable>,
        methods: Vec<Function>,
    ) -> StmtNode {
        let id = self.nodes.borrow_mut().insert_with_key(|id| {
            Box::new(Class {
                id,
                name,
                superclass,
                methods,
            })
        });
        StmtNode {
            ctx: self,
            id,
            kind: StmtKind::Class,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct StmtNode {
    ctx: &'static Context,
    id: NodeId,
    kind: StmtKind,
}

impl StmtNode {
    /// Returns a clone of this node's value, or `None` if the node is not of type `T`.
    ///
    /// # Panics
    /// Panics if the node's id has been invalidated (removed), or if the underlying
    /// stored value's id does not match this node's id.
    pub(super) fn get<T: 'static + Clone + Stmt>(&self) -> Option<T> {
        let nodes = self.ctx.nodes.borrow();
        let value = nodes[self.id].downcast_ref::<T>();

        assert!(
            value.is_none_or(|stored| stored.id() == self.id),
            "stored statement node id does not match StmtNode id"
        );

        value.cloned()
    }
}

impl Stmt for StmtNode {
    fn accept<V: Visitor>(&self, visitor: &mut V) -> V::Output {
        match &self.kind {
            // unwrap is safe here because we know the kind
            // (nodes are only created with Context, so their kind is always correct)
            StmtKind::Print => self.get::<Print>().unwrap().accept(visitor),
            StmtKind::Expression => self.get::<Expression>().unwrap().accept(visitor),
            StmtKind::Var => self.get::<Var>().unwrap().accept(visitor),
            StmtKind::Function => self.get::<Function>().unwrap().accept(visitor),
            StmtKind::Return => self.get::<Return>().unwrap().accept(visitor),
            StmtKind::If => self.get::<If>().unwrap().accept(visitor),
            StmtKind::While => self.get::<While>().unwrap().accept(visitor),
            StmtKind::Block => self.get::<Block>().unwrap().accept(visitor),
            StmtKind::Class => self.get::<Class>().unwrap().accept(visitor),
        }
    }

    fn id(&self) -> NodeId {
        self.id
    }
}

#[derive(Debug, Clone, Copy)]
pub enum StmtKind {
    Print,
    Expression,
    Var,
    Function,
    Return,
    If,
    While,
    Block,
    Class,
}

#[derive(Debug, Clone)]
pub struct Class {
    id: NodeId,
    pub name: Token,
    pub superclass: Option<Variable>,
    pub methods: Vec<Function>,
}

impl Stmt for Class {
    fn accept<V: Visitor>(&self, visitor: &mut V) -> V::Output {
        visitor.visit_class_stmt(self)
    }

    fn id(&self) -> NodeId {
        self.id
    }
}

#[derive(Debug, Clone)]
pub struct Print {
    id: NodeId,
    pub expr: ExprNode,
}

impl Stmt for Print {
    fn accept<V: Visitor>(&self, visitor: &mut V) -> V::Output {
        visitor.visit_print_stmt(self)
    }

    fn id(&self) -> NodeId {
        self.id
    }
}

#[derive(Debug, Clone)]
pub struct Expression {
    id: NodeId,
    pub expr: ExprNode,
}

impl Stmt for Expression {
    fn accept<V: Visitor>(&self, visitor: &mut V) -> V::Output {
        visitor.visit_expression_stmt(self)
    }

    fn id(&self) -> NodeId {
        self.id
    }
}

#[derive(Debug, Clone)]
pub struct Var {
    id: NodeId,
    pub name: Token,
    pub initializer: Option<ExprNode>,
}

impl Stmt for Var {
    fn accept<V: Visitor>(&self, visitor: &mut V) -> V::Output {
        visitor.visit_var_stmt(self)
    }

    fn id(&self) -> NodeId {
        self.id
    }
}

#[derive(Debug, Clone)]
pub struct Function {
    id: NodeId,
    pub name: Token,
    pub parameters: Vec<Token>,
    pub body: Vec<StmtNode>,
}

impl Stmt for Function {
    fn accept<V: Visitor>(&self, visitor: &mut V) -> V::Output {
        visitor.visit_function_stmt(self)
    }

    fn id(&self) -> NodeId {
        self.id
    }
}

#[derive(Debug, Clone)]
pub struct Return {
    id: NodeId,
    pub keyword: Token,
    pub value: Option<ExprNode>,
}

impl Stmt for Return {
    fn accept<V: Visitor>(&self, visitor: &mut V) -> V::Output {
        visitor.visit_return_stmt(self)
    }

    fn id(&self) -> NodeId {
        self.id
    }
}

#[derive(Debug, Clone)]
pub struct If {
    id: NodeId,
    pub condition: ExprNode,
    pub then_branch: StmtNode,
    pub else_branch: Option<StmtNode>,
}

impl Stmt for If {
    fn accept<V: Visitor>(&self, visitor: &mut V) -> V::Output {
        visitor.visit_if_stmt(self)
    }

    fn id(&self) -> NodeId {
        self.id
    }
}

#[derive(Debug, Clone)]
pub struct While {
    id: NodeId,
    pub condition: ExprNode,
    pub body: StmtNode,
}

impl Stmt for While {
    fn accept<V: Visitor>(&self, visitor: &mut V) -> V::Output {
        visitor.visit_while_stmt(self)
    }

    fn id(&self) -> NodeId {
        self.id
    }
}

#[derive(Debug, Clone)]
pub struct Block {
    id: NodeId,
    pub statements: Vec<StmtNode>,
}

impl Stmt for Block {
    fn accept<V: Visitor>(&self, visitor: &mut V) -> V::Output {
        visitor.visit_block_stmt(self)
    }

    fn id(&self) -> NodeId {
        self.id
    }
}
