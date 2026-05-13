use crate::Value;
use crate::parser::ast::{Context, NodeId};
use crate::scanner::token::Token;

pub trait Expr {
    fn accept<V: Visitor>(&self, v: &mut V) -> V::Output;
}

pub trait Visitor {
    type Output;
    fn visit_literal_expr(&self, expr: &Literal) -> Self::Output;
    fn visit_grouping_expr(&mut self, expr: &Grouping) -> Self::Output;
    fn visit_call_expr(&mut self, expr: &Call) -> Self::Output;
    fn visit_unary_expr(&mut self, expr: &Unary) -> Self::Output;
    fn visit_variable_expr(&self, expr: &Variable) -> Self::Output;
    fn visit_assign_expr(&mut self, expr: &Assign) -> Self::Output;
    fn visit_logical_expr(&mut self, expr: &Logical) -> Self::Output;
    fn visit_binary_expr(&mut self, expr: &Binary) -> Self::Output;
}

impl Context {
    pub(super) fn new_grouping(&'static self, expression: ExprNode) -> ExprNode {
        let node = Grouping { expression };
        let id = self.nodes.borrow_mut().insert(Box::new(node));
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
        let node = Call {
            callee,
            paren,
            arguments,
        };
        let id = self.nodes.borrow_mut().insert(Box::new(node));
        ExprNode {
            ctx: self,
            id,
            kind: ExprKind::Call,
        }
    }

    pub(super) fn new_binary(
        &'static self,
        left: ExprNode,
        operator: Token,
        right: ExprNode,
    ) -> ExprNode {
        let node = Binary {
            left,
            operator,
            right,
        };
        let id = self.nodes.borrow_mut().insert(Box::new(node));
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
        let node = Logical {
            left,
            operator,
            right,
        };
        let id = self.nodes.borrow_mut().insert(Box::new(node));
        ExprNode {
            ctx: self,
            id,
            kind: ExprKind::Logical,
        }
    }

    pub(super) fn new_unary(&'static self, operator: Token, right: ExprNode) -> ExprNode {
        let node = Unary { operator, right };
        let id = self.nodes.borrow_mut().insert(Box::new(node));
        ExprNode {
            ctx: self,
            id,
            kind: ExprKind::Unary,
        }
    }

    pub(super) fn new_variable(&'static self, name: Token) -> ExprNode {
        let node = Variable { name };
        let id = self.nodes.borrow_mut().insert(Box::new(node));
        ExprNode {
            ctx: self,
            id,
            kind: ExprKind::Variable,
        }
    }

    pub(super) fn new_assign(&'static self, name: Token, value: ExprNode) -> ExprNode {
        let node = Assign { name, value };
        let id = self.nodes.borrow_mut().insert(Box::new(node));
        ExprNode {
            ctx: self,
            id,
            kind: ExprKind::Assign,
        }
    }

    pub(super) fn new_literal(&'static self, value: Value) -> ExprNode {
        let node = Literal { value };
        let id = self.nodes.borrow_mut().insert(Box::new(node));
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
    /// Returns a cloned copy of this node's value, or `None` if the node
    /// does not exist or is not of type `T`.
    pub(super) fn get<T: 'static + Clone>(&self) -> Option<T> {
        let nodes = self.ctx.nodes.borrow();
        let value = nodes.get(self.id)?.downcast_ref::<T>().cloned();
        value
    }
}

impl Expr for ExprNode {
    fn accept<V: Visitor>(&self, v: &mut V) -> V::Output {
        let nodes = self.ctx.nodes.borrow();
        // access the node using the node id
        // this is safe if node ids are never invalidated
        // FIXME: consider using .get instead
        let node = nodes[self.id].as_ref();

        match &self.kind {
            // unwrap is safe here because we know the kind
            ExprKind::Grouping => node.downcast_ref::<Grouping>().unwrap().accept(v),
            ExprKind::Binary => node.downcast_ref::<Binary>().unwrap().accept(v),
            ExprKind::Unary => node.downcast_ref::<Unary>().unwrap().accept(v),
            ExprKind::Literal => node.downcast_ref::<Literal>().unwrap().accept(v),
            ExprKind::Variable => node.downcast_ref::<Variable>().unwrap().accept(v),
            ExprKind::Assign => node.downcast_ref::<Assign>().unwrap().accept(v),
            ExprKind::Logical => node.downcast_ref::<Logical>().unwrap().accept(v),
            ExprKind::Call => node.downcast_ref::<Call>().unwrap().accept(v),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ExprKind {
    Grouping,
    Call,
    Binary,
    Logical,
    Unary,
    Variable,
    Assign,
    Literal,
}

#[derive(Debug, Clone)]
pub struct Grouping {
    pub expression: ExprNode,
}

impl Expr for Grouping {
    fn accept<V: Visitor>(&self, v: &mut V) -> V::Output {
        v.visit_grouping_expr(self)
    }
}

#[derive(Debug, Clone)]
pub struct Call {
    pub callee: ExprNode,
    pub paren: Token,
    pub arguments: Vec<ExprNode>,
}

impl Expr for Call {
    fn accept<V: Visitor>(&self, v: &mut V) -> V::Output {
        v.visit_call_expr(self)
    }
}

#[derive(Debug, Clone)]
pub struct Binary {
    pub left: ExprNode,
    pub operator: Token,
    pub right: ExprNode,
}

impl Expr for Binary {
    fn accept<V: Visitor>(&self, v: &mut V) -> V::Output {
        v.visit_binary_expr(self)
    }
}

#[derive(Debug, Clone)]
pub struct Unary {
    pub operator: Token,
    pub right: ExprNode,
}

impl Expr for Unary {
    fn accept<V: Visitor>(&self, v: &mut V) -> V::Output {
        v.visit_unary_expr(self)
    }
}

#[derive(Debug, Clone)]
pub struct Logical {
    pub left: ExprNode,
    pub operator: Token,
    pub right: ExprNode,
}

impl Expr for Logical {
    fn accept<V: Visitor>(&self, v: &mut V) -> V::Output {
        v.visit_logical_expr(self)
    }
}

#[derive(Debug, Clone)]
pub struct Variable {
    pub name: Token,
}

impl Expr for Variable {
    fn accept<V: Visitor>(&self, v: &mut V) -> V::Output {
        v.visit_variable_expr(self)
    }
}

#[derive(Debug, Clone)]
pub struct Assign {
    pub name: Token,
    pub value: ExprNode,
}

impl Expr for Assign {
    fn accept<V: Visitor>(&self, v: &mut V) -> V::Output {
        v.visit_assign_expr(self)
    }
}

#[derive(Debug, Clone)]
pub struct Literal {
    pub value: Value,
}

impl Expr for Literal {
    fn accept<V: Visitor>(&self, v: &mut V) -> V::Output {
        v.visit_literal_expr(self)
    }
}

impl From<Value> for Literal {
    fn from(value: Value) -> Self {
        Literal { value }
    }
}
