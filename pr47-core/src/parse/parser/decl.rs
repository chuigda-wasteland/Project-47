#![allow(unused_variables)]
#![allow(dead_code)]

use crate::diag::location::{SourceLoc, SourceRange};
use super::Parser;

use crate::syntax::decl::{ConcreteExportDecl, ConcreteFuncDecl, ConcreteImportDecl, ConcreteObjectDecl, ConcreteOpenImportDecl, FunctionParam};
use crate::syntax::expr::ConcreteExpr;
use crate::syntax::id::Identifier;
use crate::syntax::token::{Token, TokenInner};
use crate::syntax::ty::ConcreteType;

impl<'s, 'd> Parser<'s, 'd> {
    pub fn parse_object_decl(&mut self, kwd_token: Token<'s>, failsafe_set: &[&[TokenInner<'_>]])
        -> Option<ConcreteObjectDecl<'s>>
    {
        let kwd_range: SourceRange = kwd_token.range;
        let id: Identifier = self.parse_unqual_ident()
            .or_else(|| { self.skip_to_any_of(failsafe_set); None } )?;
        self.skip_optional(TokenInner::SymColon);
        let (ty, eq_range): (Option<ConcreteType>, SourceRange) =
            if self.current_token().token_inner == TokenInner::SymEq {
                (None, self.consume_token().range)
            } else {
                let ty: ConcreteType = self.parse_type(failsafe_set)?;
                let eq_range: SourceRange =
                    self.expect_n_consume(TokenInner::SymEq, failsafe_set)?.range;
                (Some(ty), eq_range)
            };
        let init_expr: ConcreteExpr = self.parse_expression_no_assign(failsafe_set)?;
        self.skip_optional(TokenInner::SymSemicolon);
        Some(ConcreteObjectDecl {
            attr: None,
            name: id,
            obj_type: ty,
            init_expr,
            kwd_range,
            eq_range
        })
    }

    pub fn parse_func_decl(&mut self, kwd_token: Token<'s>, failsafe_set: &[&[TokenInner<'_>]])
        -> Option<ConcreteFuncDecl<'s>>
    {
        let func_range: SourceRange = kwd_token.range;
        let id: Identifier = self.parse_unqual_ident()
            .or_else(|| { self.skip_to_any_of(failsafe_set); None } )?;
        let lparen_loc: SourceLoc =
            self.expect_n_consume(TokenInner::SymLParen, failsafe_set)?.range.left();
        let (params, rparen_range): (Vec<FunctionParam>, SourceRange) = self.parse_list_alike(
            Self::parse_func_param,
            failsafe_set,
            TokenInner::SymComma,
            TokenInner::SymRParen,
            failsafe_set
        )?;
        todo!()
    }

    pub fn parse_func_param(&mut self, _failsafe_set: &[&[TokenInner<'_>]])
        -> Option<FunctionParam<'s>>
    {
        todo!()
    }

    pub fn parse_export_decl(&mut self, kwd_token: Token<'s>, _failsafe_set: &[&[TokenInner<'_>]])
        -> Option<ConcreteExportDecl<'s>>
    {
        todo!()
    }

    pub fn parse_import_decl(&mut self, kwd_token: Token<'s>, _failsafe_set: &[&[TokenInner<'_>]])
        -> Option<ConcreteImportDecl<'s>>
    {
        todo!()
    }

    pub fn parse_open_import_decl(
        &mut self,
        kwd_token: Token<'s>,
        _failsafe_set: &[&[TokenInner<'_>]]
    ) -> Option<ConcreteOpenImportDecl<'s>> {
        todo!()
    }
}
