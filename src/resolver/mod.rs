use std::collections::HashMap;

use slotmap::SecondaryMap;

use crate::error::StaticError;
use crate::parser::ast::NodeId;
use crate::parser::expr::{self, Expr, ExprNode};
use crate::parser::stmt::{self, Stmt, StmtNode};
use crate::scanner::token::Token;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
enum FunctionType {
    #[default]
    None,
    Function,
    Method,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
enum ClassType {
    #[default]
    None,
    Class,
}

/// A map of variable names to their resolved state.
/// The boolean value indicates whether the variable's
/// initializer has been resolved.
type Scope = HashMap<String, bool>;

#[derive(Debug, Default)]
pub struct Resolver {
    bindings: SecondaryMap<NodeId, usize>,
    scopes: Vec<Scope>,
    current_function: FunctionType,
    current_class: ClassType,
}

impl Resolver {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn resolve(
        mut self,
        stmts: &[StmtNode],
    ) -> Result<SecondaryMap<NodeId, usize>, StaticError> {
        self.resolve_body(stmts)?;
        Ok(self.bindings)
    }

    fn resolve_body(&mut self, stmts: &[StmtNode]) -> Result<(), StaticError> {
        for s in stmts {
            s.accept(self)?;
        }
        Ok(())
    }

    fn resolve_statement(&mut self, stmt: StmtNode) -> Result<(), StaticError> {
        stmt.accept(self)
    }

    fn resolve_expression(&mut self, expr: ExprNode) -> Result<(), StaticError> {
        expr.accept(self)
    }

    fn resolve_local(&mut self, expr: &impl Expr, name: &Token) {
        let Some(p) = self
            .scopes
            .iter()
            .rposition(|s| s.contains_key(&name.lexeme))
        else {
            // If we can't find the variable in any scope,
            // we leave it unresolved and assume it's global.
            return;
        };

        let distance = self.scopes.len() - 1 - p;
        self.bindings.insert(expr.id(), distance);
    }

    fn resolve_function(
        &mut self,
        fun: &stmt::Function,
        typ: FunctionType,
    ) -> Result<(), StaticError> {
        self.with_function_context(typ, |this| {
            this.with_new_scope(|this| {
                for param in fun.parameters.iter() {
                    this.declare(param)?;
                    this.define(param);
                }
                this.resolve_body(&fun.body)?;
                Ok(())
            })
        })
    }

    fn with_function_context<F, R>(&mut self, typ: FunctionType, op: F) -> Result<R, StaticError>
    where
        F: FnOnce(&mut Self) -> Result<R, StaticError>,
    {
        let enclosing = std::mem::replace(&mut self.current_function, typ);
        let result = op(self);
        self.current_function = enclosing;
        result
    }

    fn with_class_context<F, R>(&mut self, typ: ClassType, op: F) -> Result<R, StaticError>
    where
        F: FnOnce(&mut Self) -> Result<R, StaticError>,
    {
        let enclosing = std::mem::replace(&mut self.current_class, typ);
        let result = op(self);
        self.current_class = enclosing;
        result
    }

    fn with_new_scope<F, R>(&mut self, op: F) -> Result<R, StaticError>
    where
        F: FnOnce(&mut Self) -> Result<R, StaticError>,
    {
        self.begin_scope();
        let result = op(self);
        self.end_scope();
        result
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare(&mut self, name: &Token) -> Result<(), StaticError> {
        let Some(current) = self.scopes.last_mut() else {
            return Ok(());
        };

        if current.contains_key(&name.lexeme) {
            return Err(StaticError::error_at_token(
                name,
                "Already a variable with this name in this scope.",
            ));
        }

        current.insert(name.lexeme.to_owned(), false);
        Ok(())
    }

    fn define(&mut self, name: &Token) {
        let Some(current) = self.scopes.last_mut() else {
            return;
        };
        current.insert(name.lexeme.to_owned(), true);
    }
}

impl stmt::Visitor for Resolver {
    type Output = Result<(), StaticError>;

    fn visit_print_stmt(&mut self, stmt: &stmt::Print) -> Self::Output {
        self.resolve_expression(stmt.expr)
    }

    fn visit_expression_stmt(&mut self, stmt: &stmt::Expression) -> Self::Output {
        self.resolve_expression(stmt.expr)
    }

    fn visit_var_stmt(&mut self, stmt: &stmt::Var) -> Self::Output {
        self.declare(&stmt.name)?;
        if let Some(init) = stmt.initializer {
            self.resolve_expression(init)?;
        }
        self.define(&stmt.name);
        Ok(())
    }

    fn visit_function_stmt(&mut self, stmt: &stmt::Function) -> Self::Output {
        self.declare(&stmt.name)?;
        self.define(&stmt.name);
        self.resolve_function(stmt, FunctionType::Function)?;
        Ok(())
    }

    fn visit_return_stmt(&mut self, stmt: &stmt::Return) -> Self::Output {
        if self.current_function == FunctionType::None {
            return Err(StaticError::error_at_token(
                &stmt.keyword,
                "Can't return from top-level code.",
            ));
        }
        if let Some(value) = stmt.value {
            self.resolve_expression(value)?;
        }
        Ok(())
    }

    fn visit_if_stmt(&mut self, stmt: &stmt::If) -> Self::Output {
        self.resolve_expression(stmt.condition)?;
        self.resolve_statement(stmt.then_branch)?;
        if let Some(else_branch) = stmt.else_branch {
            self.resolve_statement(else_branch)?;
        }
        Ok(())
    }

    fn visit_while_stmt(&mut self, stmt: &stmt::While) -> Self::Output {
        self.resolve_expression(stmt.condition)?;
        self.resolve_statement(stmt.body)?;
        Ok(())
    }

    fn visit_block_stmt(&mut self, stmt: &stmt::Block) -> Self::Output {
        self.with_new_scope(|this| {
            this.resolve_body(&stmt.statements)?;
            Ok(())
        })
    }

    fn visit_class_stmt(&mut self, stmt: &stmt::Class) -> Self::Output {
        self.with_class_context(ClassType::Class, |this| {
            this.declare(&stmt.name)?;
            this.define(&stmt.name);
            this.with_new_scope(|this| {
                this.scopes
                    .last_mut()
                    .unwrap()
                    .insert("this".to_owned(), true);
                for method in &stmt.methods {
                    this.resolve_function(method, FunctionType::Method)?;
                }
                Ok(())
            })
        })
    }
}

impl expr::Visitor for Resolver {
    type Output = Result<(), StaticError>;

    fn visit_literal_expr(&mut self, _expr: &expr::Literal) -> Self::Output {
        Ok(())
    }

    fn visit_grouping_expr(&mut self, expr: &expr::Grouping) -> Self::Output {
        self.resolve_expression(expr.expression)
    }

    fn visit_call_expr(&mut self, expr: &expr::Call) -> Self::Output {
        self.resolve_expression(expr.callee)?;
        for arg in expr.arguments.iter() {
            self.resolve_expression(*arg)?;
        }
        Ok(())
    }

    fn visit_get_expr(&mut self, expr: &expr::Get) -> Self::Output {
        self.resolve_expression(expr.object)
    }

    fn visit_set_expr(&mut self, expr: &expr::Set) -> Self::Output {
        self.resolve_expression(expr.value)?;
        self.resolve_expression(expr.object)
    }

    fn visit_this_expr(&mut self, expr: &expr::This) -> Self::Output {
        if self.current_class == ClassType::None {
            return Err(StaticError::error_at_token(
                &expr.keyword,
                "Can't use 'this' outside of a class.",
            ));
        }
        self.resolve_local(expr, &expr.keyword);
        Ok(())
    }

    fn visit_variable_expr(&mut self, expr: &expr::Variable) -> Self::Output {
        // Check to see if the variable is being accessed inside its own initializer.
        // If the variable exists in the current scope but its value is false,
        // that means we have declared it but not yet defined it.
        if let Some(current) = self.scopes.last()
            && let Some(false) = current.get(&expr.name.lexeme)
        {
            return Err(StaticError::error_at_token(
                &expr.name,
                "Can't read local variable in its own initializer.",
            ));
        }

        self.resolve_local(expr, &expr.name);
        Ok(())
    }

    fn visit_assign_expr(&mut self, expr: &expr::Assign) -> Self::Output {
        self.resolve_expression(expr.value)?;
        self.resolve_local(expr, &expr.name);
        Ok(())
    }

    fn visit_unary_expr(&mut self, expr: &expr::Unary) -> Self::Output {
        self.resolve_expression(expr.right)
    }

    fn visit_binary_expr(&mut self, expr: &expr::Binary) -> Self::Output {
        self.resolve_expression(expr.left)?;
        self.resolve_expression(expr.right)?;
        Ok(())
    }

    fn visit_logical_expr(&mut self, expr: &expr::Logical) -> Self::Output {
        self.resolve_expression(expr.left)?;
        self.resolve_expression(expr.right)?;
        Ok(())
    }
}
