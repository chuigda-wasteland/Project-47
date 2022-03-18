use crate::diag::location::SourceRange;
use crate::syntax::decl::{ConcreteDecl, ConcreteFuncDecl, ConcreteObjectDecl};
use crate::syntax::expr::{
    ConcreteAsExpr,
    ConcreteAwaitExpr,
    ConcreteBinaryExpr,
    ConcreteExpr,
    ConcreteFieldRefExpr,
    ConcreteFuncCallExpr,
    ConcreteLiteralExpr,
    ConcreteParenthesizedExpr,
    ConcreteSubscriptExpr,
    ConcreteUnaryExpr
};
use crate::syntax::id::Identifier;
use crate::syntax::stmt::{ConcreteCompoundStmt, ConcreteStmt};
use crate::syntax::token::Token;
use crate::syntax::ty::{ConcreteGenericType, ConcreteNullableType, ConcreteType};

pub trait DeclVisitor<'s> {
    type DeclResult;

    fn visit_decl(&mut self, decl: &'s ConcreteDecl<'s>) -> Option<Self::DeclResult> {
        Some(match decl {
            ConcreteDecl::ConstDecl(const_decl) => self.visit_const_decl(const_decl),
            ConcreteDecl::FuncDecl(func_decl) => self.visit_func_decl(func_decl),
            ConcreteDecl::VarDecl(var_decl) => self.visit_var_decl(var_decl),

            // imports and exports are resolved by other parts, so ignore them.
            _ => return None
        })
    }

    fn visit_const_decl(&mut self, const_decl: &'s ConcreteObjectDecl<'s>) -> Self::DeclResult;
    fn visit_func_decl(&mut self, func_decl: &'s ConcreteFuncDecl<'s>) -> Self::DeclResult;
    fn visit_var_decl(&mut self, var_decl: &'s ConcreteObjectDecl<'s>) -> Self::DeclResult;
}

pub trait ExprVisitor<'s> {
    type ExprResult;

    fn visit_expr(&mut self, expr: &'s ConcreteExpr<'s>) -> Self::ExprResult {
        match expr {
            ConcreteExpr::LiteralExpr(literal_expr) => self.visit_literal_expr(literal_expr),
            ConcreteExpr::IdRefExpr(id) => self.visit_id_ref_expr(id),
            ConcreteExpr::UnaryExpr(unary_expr) => self.visit_unary_expr(unary_expr),
            ConcreteExpr::BinaryExpr(binary_expr) => self.visit_binary_expr(binary_expr),
            ConcreteExpr::FuncCallExpr(func_call_expr) => self.visit_func_call_expr(func_call_expr),
            ConcreteExpr::SubscriptExpr(subscript_expr) => self.visit_subscript_expr(subscript_expr),
            ConcreteExpr::FieldRefExpr(field_ref_expr) => self.visit_field_ref_expr(field_ref_expr),
            ConcreteExpr::AsExpr(as_expr) => self.visit_as_expr(as_expr),
            ConcreteExpr::AwaitExpr(await_expr) => self.visit_await_expr(await_expr),
            ConcreteExpr::ParenthesizedExpr(paren_expr) => self.visit_parenthesized_expr(paren_expr)
        }
    }

    fn visit_literal_expr(
        &mut self,
        literal_expr: &'s ConcreteLiteralExpr<'s>
    ) -> Self::ExprResult;
    fn visit_id_ref_expr(&mut self, id: &'s Identifier<'s>) -> Self::ExprResult;
    fn visit_unary_expr(
        &mut self,
        unary_expr: &'s ConcreteUnaryExpr<'s>
    ) -> Self::ExprResult;
    fn visit_binary_expr(&mut self, binary_expr: &'s ConcreteBinaryExpr<'s>) -> Self::ExprResult;
    fn visit_func_call_expr(
        &mut self,
        func_call_expr: &'s ConcreteFuncCallExpr<'s>
    ) -> Self::ExprResult;
    fn visit_subscript_expr(
        &mut self,
        subscript_expr: &'s ConcreteSubscriptExpr<'s>
    ) -> Self::ExprResult;
    fn visit_field_ref_expr(
        &mut self,
        field_ref_expr: &'s ConcreteFieldRefExpr<'s>
    ) -> Self::ExprResult;
    fn visit_as_expr(&mut self, as_expr: &'s ConcreteAsExpr<'s>) -> Self::ExprResult;
    fn visit_await_expr(&mut self, await_expr: &'s ConcreteAwaitExpr<'s>) -> Self::ExprResult;
    fn visit_parenthesized_expr(
        &mut self,
        paren_expr: &'s ConcreteParenthesizedExpr<'s>
    ) -> Self::ExprResult {
        self.visit_expr(&paren_expr.inner)
    }
}

pub trait StmtVisitor<'s> {
    type StmtResult;

    fn visit_stmt(&mut self, stmt: &'s ConcreteStmt<'s>) -> Self::StmtResult {
        match stmt {
            ConcreteStmt::CompoundStmt(compound_stmt) => self.visit_compound_stmt(compound_stmt),
            ConcreteStmt::ExprStmt(expr_stmt, _) => self.visit_expr_stmt(expr_stmt),
            ConcreteStmt::DeclStmt(decl_stmt, _) => self.visit_decl_stmt(decl_stmt)
        }
    }

    fn visit_compound_stmt(
        &mut self,
        compound_stmt: &'s ConcreteCompoundStmt<'s>
    ) -> Self::StmtResult;

    fn visit_expr_stmt(&mut self, expr: &'s ConcreteExpr<'s>) -> Self::StmtResult;
    fn visit_decl_stmt(&mut self, decl: &'s ConcreteDecl<'s>) -> Self::StmtResult;
}

pub trait TypeVisitor<'s> {
    type TypeResult;

    fn visit_type(&mut self, ty: &'s ConcreteType<'s>) -> Self::TypeResult {
        match ty {
            ConcreteType::PrimitiveType(primitive_type) => self.visit_primitive_type(primitive_type),
            ConcreteType::GenericType(generic_type) => self.visit_generic_type(generic_type),
            ConcreteType::NullableType(nullable_type) => self.visit_nullable_type(nullable_type),
            ConcreteType::DeducedType(source_range) => self.visit_deduced_type(*source_range),
            ConcreteType::UserType(id) => self.visit_user_type(id)
        }
    }

    fn visit_primitive_type(&mut self, primitive_type: &'s Token<'s>) -> Self::TypeResult;
    fn visit_generic_type(&mut self, generic_type: &'s ConcreteGenericType<'s>) -> Self::TypeResult;
    fn visit_nullable_type(&mut self, nullable_type: &'s ConcreteNullableType<'s>) -> Self::TypeResult;
    fn visit_deduced_type(&mut self, deduced_type_source_range: SourceRange) -> Self::TypeResult;
    fn visit_user_type(&mut self, user_type: &'s Identifier<'s>) -> Self::TypeResult;
}
