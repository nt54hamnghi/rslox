use crate::Value;
use crate::parser::ast::{Context, NodeId};
use crate::scanner::token::Token;

pub trait Expr {
    fn accept<V: Visitor>(&self, visitor: &mut V) -> V::Output;
    fn id(&self) -> NodeId;
}

pub trait Visitor {
    type Output;
    fn visit_literal_expr(&mut self, expr: &Literal) -> Self::Output;
    fn visit_grouping_expr(&mut self, expr: &Grouping) -> Self::Output;
    fn visit_call_expr(&mut self, expr: &Call) -> Self::Output;
    fn visit_get_expr(&mut self, expr: &Get) -> Self::Output;
    fn visit_unary_expr(&mut self, expr: &Unary) -> Self::Output;
    fn visit_variable_expr(&mut self, expr: &Variable) -> Self::Output;
    fn visit_assign_expr(&mut self, expr: &Assign) -> Self::Output;
    fn visit_logical_expr(&mut self, expr: &Logical) -> Self::Output;
    fn visit_binary_expr(&mut self, expr: &Binary) -> Self::Output;
}

impl Context {
    pub(super) fn new_grouping(&'static self, expression: ExprNode) -> ExprNode {
        let id = self
            .nodes
            .borrow_mut()
            .insert_with_key(|id| Box::new(Grouping { id, expression }));
        ExprNode {
            ctx: self,
            id,
            kind: ExprKind::Grouping,
        }
    }

    pub(super) fn new_call(
        &'static self,
        callee: ExprNode,
        paren: Token,
        arguments: Vec<ExprNode>,
    ) -> ExprNode {
        let id = self.nodes.borrow_mut().insert_with_key(|id| {
            Box::new(Call {
                id,
                callee,
                paren,
                arguments,
            })
        });
        ExprNode {
            ctx: self,
            id,
            kind: ExprKind::Call,
        }
    }

    pub(super) fn new_get(&'static self, object: ExprNode, name: Token) -> ExprNode {
        let id = self
            .nodes
            .borrow_mut()
            .insert_with_key(|id| Box::new(Get { id, object, name }));
        ExprNode {
            ctx: self,
            id,
            kind: ExprKind::Get,
        }
    }

    pub(super) fn new_binary(
        &'static self,
        left: ExprNode,
        operator: Token,
        right: ExprNode,
    ) -> ExprNode {
        let id = self.nodes.borrow_mut().insert_with_key(|id| {
            Box::new(Binary {
                id,
                left,
                operator,
                right,
            })
        });
        ExprNode {
            ctx: self,
            id,
            kind: ExprKind::Binary,
        }
    }

    pub(super) fn new_logical(
        &'static self,
        left: ExprNode,
        operator: Token,
        right: ExprNode,
    ) -> ExprNode {
        let id = self.nodes.borrow_mut().insert_with_key(|id| {
            Box::new(Logical {
                id,
                left,
                operator,
                right,
            })
        });
        ExprNode {
            ctx: self,
            id,
            kind: ExprKind::Logical,
        }
    }

    pub(super) fn new_unary(&'static self, operator: Token, right: ExprNode) -> ExprNode {
        let id = self.nodes.borrow_mut().insert_with_key(|id| {
            Box::new(Unary {
                id,
                operator,
                right,
            })
        });
        ExprNode {
            ctx: self,
            id,
            kind: ExprKind::Unary,
        }
    }

    pub(super) fn new_variable(&'static self, name: Token) -> ExprNode {
        let id = self
            .nodes
            .borrow_mut()
            .insert_with_key(|id| Box::new(Variable { id, name }));
        ExprNode {
            ctx: self,
            id,
            kind: ExprKind::Variable,
        }
    }

    pub(super) fn new_assign(&'static self, name: Token, value: ExprNode) -> ExprNode {
        let id = self
            .nodes
            .borrow_mut()
            .insert_with_key(|id| Box::new(Assign { id, name, value }));
        ExprNode {
            ctx: self,
            id,
            kind: ExprKind::Assign,
        }
    }

    pub(super) fn new_literal(&'static self, value: Value) -> ExprNode {
        let id = self
            .nodes
            .borrow_mut()
            .insert_with_key(|id| Box::new(Literal { id, value }));
        ExprNode {
            ctx: self,
            id,
            kind: ExprKind::Literal,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ExprNode {
    ctx: &'static Context,
    id: NodeId,
    kind: ExprKind,
}

impl ExprNode {
    /// Returns a clone of this node's value, or `None` if the node is not of type `T`.
    ///
    /// # Panics
    /// Panics if the node's id has been invalidated (removed), or if the underlying
    /// stored value's id does not match this node's id.
    pub(super) fn get<T: 'static + Clone + Expr>(&self) -> Option<T> {
        let nodes = self.ctx.nodes.borrow();
        let value = nodes[self.id].downcast_ref::<T>().cloned();

        assert!(
            value.as_ref().is_none_or(|v| v.id() == self.id),
            "stored expression node id does not match ExprNode id"
        );

        value
    }
}

impl Expr for ExprNode {
    fn accept<V: Visitor>(&self, visitor: &mut V) -> V::Output {
        match &self.kind {
            // unwrap is safe here because we know the kind
            // (nodes are only created with Context, so their kind is always correct)
            ExprKind::Grouping => self.get::<Grouping>().unwrap().accept(visitor),
            ExprKind::Binary => self.get::<Binary>().unwrap().accept(visitor),
            ExprKind::Unary => self.get::<Unary>().unwrap().accept(visitor),
            ExprKind::Literal => self.get::<Literal>().unwrap().accept(visitor),
            ExprKind::Variable => self.get::<Variable>().unwrap().accept(visitor),
            ExprKind::Assign => self.get::<Assign>().unwrap().accept(visitor),
            ExprKind::Logical => self.get::<Logical>().unwrap().accept(visitor),
            ExprKind::Call => self.get::<Call>().unwrap().accept(visitor),
            ExprKind::Get => self.get::<Get>().unwrap().accept(visitor),
        }
    }

    fn id(&self) -> NodeId {
        self.id
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ExprKind {
    Grouping,
    Call,
    Get,
    Binary,
    Logical,
    Unary,
    Variable,
    Assign,
    Literal,
}

#[derive(Debug, Clone)]
pub struct Grouping {
    id: NodeId,
    pub expression: ExprNode,
}

impl Expr for Grouping {
    fn accept<V: Visitor>(&self, visitor: &mut V) -> V::Output {
        visitor.visit_grouping_expr(self)
    }

    fn id(&self) -> NodeId {
        self.id
    }
}

#[derive(Debug, Clone)]
pub struct Call {
    id: NodeId,
    pub callee: ExprNode,
    pub paren: Token,
    pub arguments: Vec<ExprNode>,
}

impl Expr for Call {
    fn accept<V: Visitor>(&self, visitor: &mut V) -> V::Output {
        visitor.visit_call_expr(self)
    }

    fn id(&self) -> NodeId {
        self.id
    }
}

#[derive(Debug, Clone)]
pub struct Get {
    id: NodeId,
    pub object: ExprNode,
    pub name: Token,
}

impl Expr for Get {
    fn accept<V: Visitor>(&self, visitor: &mut V) -> V::Output {
        visitor.visit_get_expr(self)
    }

    fn id(&self) -> NodeId {
        self.id
    }
}

#[derive(Debug, Clone)]
pub struct Binary {
    id: NodeId,
    pub left: ExprNode,
    pub operator: Token,
    pub right: ExprNode,
}

impl Expr for Binary {
    fn accept<V: Visitor>(&self, visitor: &mut V) -> V::Output {
        visitor.visit_binary_expr(self)
    }

    fn id(&self) -> NodeId {
        self.id
    }
}

#[derive(Debug, Clone)]
pub struct Unary {
    id: NodeId,
    pub operator: Token,
    pub right: ExprNode,
}

impl Expr for Unary {
    fn accept<V: Visitor>(&self, visitor: &mut V) -> V::Output {
        visitor.visit_unary_expr(self)
    }

    fn id(&self) -> NodeId {
        self.id
    }
}

#[derive(Debug, Clone)]
pub struct Logical {
    id: NodeId,
    pub left: ExprNode,
    pub operator: Token,
    pub right: ExprNode,
}

impl Expr for Logical {
    fn accept<V: Visitor>(&self, visitor: &mut V) -> V::Output {
        visitor.visit_logical_expr(self)
    }

    fn id(&self) -> NodeId {
        self.id
    }
}

#[derive(Debug, Clone)]
pub struct Variable {
    id: NodeId,
    pub name: Token,
}

impl Expr for Variable {
    fn accept<V: Visitor>(&self, visitor: &mut V) -> V::Output {
        visitor.visit_variable_expr(self)
    }

    fn id(&self) -> NodeId {
        self.id
    }
}

#[derive(Debug, Clone)]
pub struct Assign {
    id: NodeId,
    pub name: Token,
    pub value: ExprNode,
}

impl Expr for Assign {
    fn accept<V: Visitor>(&self, visitor: &mut V) -> V::Output {
        visitor.visit_assign_expr(self)
    }

    fn id(&self) -> NodeId {
        self.id
    }
}

#[derive(Debug, Clone)]
pub struct Literal {
    id: NodeId,
    pub value: Value,
}

impl Expr for Literal {
    fn accept<V: Visitor>(&self, visitor: &mut V) -> V::Output {
        visitor.visit_literal_expr(self)
    }

    fn id(&self) -> NodeId {
        self.id
    }
}
