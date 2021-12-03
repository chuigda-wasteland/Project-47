use super::{Parser, TOP_LEVEL_DECL_FAILSAFE};

use unchecked_unwrap::UncheckedUnwrap;
use xjbutil::either::Either;

use crate::awa;
use crate::diag::diag_data;
use crate::diag::location::SourceRange;
use crate::parse::lexer::LexerMode;
use crate::parse::parser::TOP_LEVEL_FIRST;
use crate::syntax::ConcreteProgram;
use crate::syntax::attr::Attribute;
use crate::syntax::decl::{ConcreteDecl, ConcreteExportDecl, ConcreteFuncDecl, ConcreteImportDecl, ConcreteObjectDecl, ConcreteOpenImportDecl};
use crate::syntax::token::{Token, TokenInner};

impl<'s, 'd> Parser<'s, 'd> {
    pub fn parse(&mut self) -> ConcreteProgram<'s> {
        self.lexer.push_lexer_mode(LexerMode::LexTopDecl);

        let mut program = ConcreteProgram::new();
        while !self.m_current_token.is_eoi() {
            if self.m_current_token.token_inner == TokenInner::SymHash {
                let hash_token: Token<'s> = self.consume_token();
                if let Some(item) = self.parse_global_attr_or_attributed_decl(hash_token) {
                    match item {
                        Either::Left(decl) => program.decls.push(decl),
                        Either::Right(global_attr) => program.global_attrs.push(global_attr)
                    }
                }
            } else {
                if let Some(top_level_decl) = self.parse_top_level_decl() {
                    program.decls.push(top_level_decl);
                }
            }
        }

        program
    }

    pub fn parse_global_attr_or_attributed_decl(&mut self, hash_token: Token<'s>)
        -> Option<Either<ConcreteDecl<'s>, Attribute<'s>>>
    {
        #[cfg(debug_assertions)] assert_eq!(hash_token.token_inner, TokenInner::SymHash);

        match self.current_token().token_inner {
            TokenInner::SymLBracket => {
                self.parse_attributed_top_level_decl(hash_token)
                    .map(|attributed_decl| Either::Left(attributed_decl))
            },
            TokenInner::SymExclaim => {
                self.parse_attribute(hash_token, true, TOP_LEVEL_FIRST)
                    .map(|global_attr| Either::Right(global_attr))
            },
            _ => {
                self.diag.borrow_mut()
                    .diag(self.current_token().range.left(),
                          diag_data::err_expected_any_of_0_got_1)
                    .add_arg2(awa![TokenInner::SymExclaim, TokenInner::SymLBracket])
                    .add_arg2(self.current_token().token_inner)
                    .add_mark(self.current_token().range.into())
                    .emit();
                None
            }
        }
    }

    pub fn parse_attributed_top_level_decl(
        &mut self,
        hash_token: Token<'s>
    ) -> Option<ConcreteDecl<'s>> {
        #[cfg(debug_assertions)] assert_eq!(hash_token.token_inner, TokenInner::SymHash);

        let attr_list: Attribute = self.parse_attribute(hash_token, false, TOP_LEVEL_FIRST)?;
        let mut decl: ConcreteDecl = self.parse_top_level_decl()?;

        match &mut decl {
            ConcreteDecl::ConstDecl(const_decl) => unsafe {
                const_decl.attr.replace(attr_list).unchecked_unwrap();
            },
            ConcreteDecl::FuncDecl(func_decl) => unsafe {
                func_decl.attr.replace(attr_list).unchecked_unwrap();
            },
            ConcreteDecl::ExportDecl(export_decl) => {
                self.diag.borrow_mut()
                    .diag(export_decl.export_kwd_range.left(),
                          diag_data::err_export_decl_disallow_attr)
                    .add_mark(export_decl.export_kwd_range.into())
                    .emit();
            },
            ConcreteDecl::ImportDecl(import_decl) => {
                self.diag.borrow_mut()
                    .diag(import_decl.import_kwd_range.left(),
                          diag_data::err_import_decl_disallow_attr)
                    .add_mark(import_decl.import_kwd_range.into())
                    .emit();
            },
            ConcreteDecl::OpenImportDecl(open_import_decl) => {
                self.diag.borrow_mut()
                    .diag(open_import_decl.open_kwd_range.left(),
                          diag_data::err_import_decl_disallow_attr)
                    .add_mark(open_import_decl.open_kwd_range.into())
                    .emit();
            },
            ConcreteDecl::VarDecl(_) => {
                unreachable!("variable declarations cannot appear at top level")
            },
        }
        Some(decl)
    }

    pub fn parse_top_level_decl(&mut self) -> Option<ConcreteDecl<'s>> {
        match self.current_token().token_inner {
            TokenInner::KwdConst => {
                let const_token: Token<'s> = self.consume_token();
                self.parse_object_decl(const_token, TOP_LEVEL_DECL_FAILSAFE)
                    .map(|const_decl: ConcreteObjectDecl| ConcreteDecl::ConstDecl(const_decl))
            },
            TokenInner::KwdVar => {
                self.diag.borrow_mut()
                    .diag(self.current_token().range.left(),
                          diag_data::err_no_top_level_var_decl)
                    .add_mark(self.current_token().range.into())
                    .emit();
                None
            },
            TokenInner::KwdFunc => {
                let func_token: Token<'s> = self.consume_token();
                self.parse_func_decl(func_token, TOP_LEVEL_DECL_FAILSAFE)
                    .map(|func_decl: ConcreteFuncDecl| ConcreteDecl::FuncDecl(func_decl))
            },
            TokenInner::KwdExport => {
                let export_token: Token<'s> = self.consume_token();
                self.parse_export_decl(export_token, TOP_LEVEL_DECL_FAILSAFE)
                    .map(|export_decl: ConcreteExportDecl| ConcreteDecl::ExportDecl(export_decl))
            },
            TokenInner::KwdImport => {
                let import_token: Token<'s> = self.consume_token();
                self.parse_import_decl(import_token, TOP_LEVEL_DECL_FAILSAFE)
                    .map(|import_decl: ConcreteImportDecl| ConcreteDecl::ImportDecl(import_decl))
            },
            TokenInner::KwdOpen => {
                let open_token: Token<'s> = self.consume_token();
                self.parse_open_import_decl(open_token, TOP_LEVEL_DECL_FAILSAFE)
                    .map(|open_import_decl: ConcreteOpenImportDecl| {
                        ConcreteDecl::OpenImportDecl(open_import_decl)
                    })
            },
            _ => {
                self.diag.borrow_mut()
                    .diag(self.current_token().range.left(), diag_data::err_expected_any_of_0_got_1)
                    .add_arg2(awa![
                        TokenInner::KwdConst,
                        TokenInner::KwdFunc,
                        TokenInner::KwdExport,
                        TokenInner::KwdImport,
                        TokenInner::KwdOpen
                    ])
                    .add_arg2(self.current_token().token_inner)
                    .add_mark(self.current_token().range.into())
                    .emit();
                self.skip_to_any_of(TOP_LEVEL_DECL_FAILSAFE);
                None
            }
        }
    }

    pub fn diag_unexpected_eoi(&mut self, range: SourceRange) {
        self.diag.borrow_mut()
            .diag(range.left(), diag_data::err_unexpected_eoi)
            .emit()
    }
}
