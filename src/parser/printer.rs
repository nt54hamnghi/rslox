use crate::parser::expr::{Binary, Expr, Grouping, Literal, Unary, Visitor};

#[derive(Clone, Copy)]
pub struct AstPrinter;

impl AstPrinter {
    pub fn print<E: Expr>(mut self, expr: &E) -> String {
        expr.accept(&mut self)
    }
}

macro_rules! parenthesize {
    ($visitor:ident, $name:expr, $($expression:expr),+) => {{
        let mut output = format!("({}", $name);
        $(
            output.push(' ');
            output.push_str(&$expression.accept($visitor));
        )+
        output.push(')');
        output
    }};
}

impl Visitor for AstPrinter {
    type Output = String;

    fn visit_literal_expr(&mut self, expr: &Literal) -> Self::Output {
        format!("{:?}", expr.value)
    }

    fn visit_grouping_expr(&mut self, expr: &Grouping) -> Self::Output {
        let Grouping { expression, .. } = expr;
        parenthesize!(self, "group", expression)
    }

    fn visit_call_expr(&mut self, _expr: &super::expr::Call) -> Self::Output {
        todo!()
    }

    fn visit_get_expr(&mut self, _expr: &super::expr::Get) -> Self::Output {
        todo!()
    }

    fn visit_set_expr(&mut self, _expr: &super::expr::Set) -> Self::Output {
        todo!()
    }

    fn visit_super_expr(&mut self, _expr: &super::expr::Super) -> Self::Output {
        todo!()
    }

    fn visit_this_expr(&mut self, _expr: &super::expr::This) -> Self::Output {
        todo!()
    }

    fn visit_variable_expr(&mut self, _expr: &super::expr::Variable) -> Self::Output {
        todo!()
    }

    fn visit_assign_expr(&mut self, _expr: &super::expr::Assign) -> Self::Output {
        todo!()
    }

    fn visit_unary_expr(&mut self, expr: &Unary) -> Self::Output {
        let Unary {
            operator, right, ..
        } = expr;
        parenthesize!(self, operator.lexeme, right)
    }

    fn visit_binary_expr(&mut self, expr: &Binary) -> Self::Output {
        let Binary {
            left,
            operator,
            right,
            ..
        } = expr;
        parenthesize!(self, operator.lexeme, left, right)
    }

    fn visit_logical_expr(&mut self, _expr: &super::expr::Logical) -> Self::Output {
        todo!()
    }
}
