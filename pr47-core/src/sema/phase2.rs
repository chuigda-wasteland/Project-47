use std::ptr::NonNull;

use xjbutil::value::Value;

use crate::data::tyck::{TyckInfo, TyckInfoPool};
use crate::diag::{diag_data, DiagContext};
use crate::sema::arena::{Arena, ArenaPtr};
use crate::sema::decl::ObjectDecl;
use crate::sema::expr::{Expr, IdRefExpr, LiteralExpr};
use crate::sema::scope::{Scope, ScopeKind};
use crate::syntax::expr::{
    ConcreteAsExpr,
    ConcreteAwaitExpr,
    ConcreteBinaryExpr,
    ConcreteFieldRefExpr,
    ConcreteFuncCallExpr,
    ConcreteLiteralExpr,
    ConcreteSubscriptExpr,
    ConcreteUnaryExpr,
    LiteralExprContent
};
use crate::syntax::id::Identifier;
use crate::syntax::visitor::ExprVisitor;

pub struct SemaPhase2<'s, 'd> {
    scope: Scope<'s>,
    arena: &'s mut Arena<'s>,
    tyck_info_pool: &'s mut TyckInfoPool,

    diag: &'d mut DiagContext
}

#[allow(unused)]
impl<'s, 'd> ExprVisitor<'s> for SemaPhase2<'s, 'd> {
    type ExprResult = Option<Expr<'s>>;

    fn visit_literal_expr(
        &mut self,
        literal_expr: &'s ConcreteLiteralExpr<'s>
    ) -> Self::ExprResult {
        let ty: NonNull<TyckInfo> = match literal_expr.content {
            LiteralExprContent::Int(_) => self.tyck_info_pool.get_int_type(),
            LiteralExprContent::Float(_) => self.tyck_info_pool.get_float_type(),
            LiteralExprContent::Char(_) => self.tyck_info_pool.get_char_type(),
            LiteralExprContent::String(_) => self.tyck_info_pool.get_string_type(),
            LiteralExprContent::Boolean(_) => self.tyck_info_pool.get_bool_type(),
        };

        let literal_expr: ArenaPtr<'s, LiteralExpr<'s>> = ArenaPtr::new_in(LiteralExpr {
            content: literal_expr.content,
            ty,
            concrete: &literal_expr
        }, &self.arena);
        Some(Expr::LiteralExpr(literal_expr))
    }

    fn visit_id_ref_expr(&mut self, id: &'s Identifier<'s>) -> Self::ExprResult {
        todo!()
    }

    fn visit_unary_expr(&mut self, unary_expr: &'s ConcreteUnaryExpr<'s>) -> Self::ExprResult {
        todo!()
    }

    fn visit_binary_expr(&mut self, binary_expr: &'s ConcreteBinaryExpr<'s>) -> Self::ExprResult {
        todo!()
    }

    fn visit_func_call_expr(&mut self, func_call_expr: &'s ConcreteFuncCallExpr<'s>) -> Self::ExprResult {
        todo!()
    }

    fn visit_subscript_expr(&mut self, subscript_expr: &'s ConcreteSubscriptExpr<'s>) -> Self::ExprResult {
        todo!()
    }

    fn visit_field_ref_expr(&mut self, field_ref_expr: &'s ConcreteFieldRefExpr<'s>) -> Self::ExprResult {
        todo!()
    }

    fn visit_as_expr(&mut self, as_expr: &'s ConcreteAsExpr<'s>) -> Self::ExprResult {
        todo!()
    }

    fn visit_await_expr(&mut self, await_expr: &'s ConcreteAwaitExpr<'s>) -> Self::ExprResult {
        todo!()
    }
}
