use super::{Parser, TOP_LEVEL_DECL_FAILSAFE};

use unchecked_unwrap::UncheckedUnwrap;
use xjbutil::either::Either;

use crate::diag::diag_data;
use crate::diag::location::SourceRange;
use crate::syntax::ConcreteProgram;
use crate::syntax::attr::Attribute;
use crate::syntax::decl::{ConcreteDecl, ConcreteObjectDecl};
use crate::syntax::token::TokenInner;

impl<'s, 'd> Parser<'s, 'd> {
    pub fn parse(&mut self) -> ConcreteProgram<'s> {
        let mut program = ConcreteProgram::new();
        while !self.m_current_token.is_eoi() {
            if self.m_current_token.token_inner == TokenInner::SymHash {
                if let Some(item) = self.parse_global_attr_or_attributed_decl() {
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

    pub fn parse_global_attr_or_attributed_decl(&mut self)
        -> Option<Either<ConcreteDecl<'s>, Attribute<'s>>>
    {
        match self.peek_token().token_inner {
            TokenInner::SymLBracket => {
                self.parse_attributed_top_level_decl()
                    .map(|attributed_decl| Either::Left(attributed_decl))
            },
            TokenInner::SymExclaim => {
                self.parse_attribute(true).map(|global_attr| Either::Right(global_attr))
            },
            _ => {
                todo!("error reporting")
            }
        }
    }

    pub fn parse_attributed_top_level_decl(&mut self) -> Option<ConcreteDecl<'s>> {
        let attr_list: Attribute = self.parse_attribute(false)?;
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
                    .build();
            },
            ConcreteDecl::ImportDecl(import_decl) => {
                self.diag.borrow_mut()
                    .diag(import_decl.import_kwd_range.left(),
                          diag_data::err_import_decl_disallow_attr)
                    .add_mark(import_decl.import_kwd_range.into())
                    .build();
            },
            ConcreteDecl::OpenImportDecl(open_import_decl) => {
                self.diag.borrow_mut()
                    .diag(open_import_decl.open_kwd_range.left(),
                          diag_data::err_import_decl_disallow_attr)
                    .add_mark(open_import_decl.open_kwd_range.into())
                    .build();
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
                self.parse_object_decl(TOP_LEVEL_DECL_FAILSAFE)
                    .map(|const_decl: ConcreteObjectDecl| ConcreteDecl::ConstDecl(const_decl))
            },
            TokenInner::KwdVar => {
                self.diag.borrow_mut()
                    .diag(self.current_token().range.left(),
                          diag_data::err_no_top_level_var_decl)
                    .add_mark(self.current_token().range.into())
                    .build();
                None
            },
            _ => todo!()
        }
    }

    pub fn diag_unexpected_eoi(&mut self, range: SourceRange) {
        self.diag.borrow_mut()
            .diag(range.left(), diag_data::err_unexpected_eoi)
            .build()
    }
}
