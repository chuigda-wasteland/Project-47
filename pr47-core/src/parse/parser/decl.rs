#![allow(unused_variables)]
#![allow(dead_code)]

use super::Parser;

use crate::awa;
use crate::diag::diag_data;
use crate::diag::location::{SourceLoc, SourceRange};
use crate::syntax::attr::Attribute;
use crate::syntax::decl::{ConcreteExportDecl, ConcreteFuncDecl, ConcreteImportDecl, ConcreteObjectDecl, ConcreteOpenImportDecl, FunctionParam, OpenImportUsingItem};
use crate::syntax::expr::ConcreteExpr;
use crate::syntax::id::Identifier;
use crate::syntax::stmt::ConcreteCompoundStmt;
use crate::syntax::token::{Token, TokenInner};
use crate::syntax::ty::ConcreteType;

impl<'s, 'd> Parser<'s, 'd> {
    pub fn parse_object_decl(&mut self, kwd_token: Token<'s>, failsafe_set: &[&[TokenInner<'_>]])
        -> Option<ConcreteObjectDecl<'s>>
    {
        let kwd_range: SourceRange = kwd_token.range;
        let id: Identifier = self.parse_unqual_ident()
            .or_else(|| { self.skip_to_any_of(failsafe_set); None } )?;
        let has_colon: bool = self.skip_optional(TokenInner::SymColon);
        let (obj_type, eq_range): (Option<ConcreteType>, SourceRange) =
            if self.current_token().token_inner == TokenInner::SymEq {
                if has_colon {
                    self.diag.borrow_mut()
                        .diag(self.current_token().range.left(), diag_data::err_missing_type_got_0)
                        .add_arg2(self.current_token().token_inner)
                        .add_mark(self.current_token().range.into())
                        .emit();
                    self.skip_to_any_of(failsafe_set);
                    return None;
                }
                (None, self.consume_token().range)
            } else {
                let ty: ConcreteType = self.parse_type(failsafe_set)?;
                let eq_range: SourceRange =
                    self.expect_n_consume(TokenInner::SymEq, failsafe_set)?.range;
                (Some(ty), eq_range)
            };
        let init_expr: ConcreteExpr = self.parse_expression_no_assign(failsafe_set)?;
        self.expect_n_consume(TokenInner::SymSemicolon, failsafe_set)?;
        Some(ConcreteObjectDecl {
            attr: None,
            name: id,
            obj_type,
            init_expr,
            kwd_range,
            eq_range
        })
    }

    pub fn parse_func_decl(&mut self, kwd_token: Token<'s>, failsafe_set: &[&[TokenInner<'_>]])
        -> Option<ConcreteFuncDecl<'s>>
    {
        let func_kwd_range: SourceRange = kwd_token.range;
        let func_name: Identifier = self.parse_unqual_ident()
            .or_else(|| { self.skip_to_any_of(failsafe_set); None })?;
        let lparen_loc: SourceLoc =
            self.expect_n_consume(TokenInner::SymLParen, failsafe_set)?.range.left();
        let (func_param_list, rparen_range): (Vec<FunctionParam>, SourceRange) =
            self.parse_list_alike(
                Self::parse_func_param,
                failsafe_set,
                TokenInner::SymComma,
                TokenInner::SymRParen,
                failsafe_set
            )?;

        let has_colon: bool = self.skip_optional(TokenInner::SymColon);
        let func_return_types: Vec<ConcreteType<'s>> =
            if self.current_token().token_inner == TokenInner::SymLBrace ||
                self.current_token().token_inner == TokenInner::SymSemicolon
            {
                if has_colon {
                    self.diag.borrow_mut()
                        .diag(self.current_token().range.left(), diag_data::err_missing_type_got_0)
                        .add_arg2(self.current_token().token_inner)
                        .add_mark(self.current_token().range.into())
                        .emit();
                    self.skip_to_any_of(failsafe_set);
                    return None;
                }
                Vec::new()
            } else {
                self.parse_func_ret_type(failsafe_set)?
            };

        let func_body: Option<ConcreteCompoundStmt<'s>> =
            if self.current_token().token_inner == TokenInner::SymSemicolon {
                let _ = self.consume_token();
                None
            } else {
                let lbrace_token: Token<'s> = self.consume_token();
                self.parse_compound_stmt(lbrace_token, failsafe_set)
            };

        Some(ConcreteFuncDecl {
            attr: None,
            func_name,
            func_param_list,
            func_return_types,
            exception_spec: None,
            func_body,
            func_kwd_range,
            param_open_paren_loc: lparen_loc,
            param_close_paren_loc: rparen_range.left()
        })
    }

    pub fn parse_func_param(&mut self, failsafe_set: &[&[TokenInner<'_>]])
        -> Option<FunctionParam<'s>>
    {
        let attr: Option<Attribute> = if self.current_token().token_inner == TokenInner::SymHash {
            let hash_token: Token<'s> = self.consume_token();
            Some(self.parse_attribute(hash_token, false, failsafe_set)?)
        } else {
            None
        };
        let param_name: Identifier = self.parse_unqual_ident()
            .or_else(|| { self.skip_to_any_of(failsafe_set); None })?;
        let has_colon: bool = self.skip_optional(TokenInner::SymColon);
        let param_type: Option<ConcreteType> =
            if self.current_token().token_inner == TokenInner::SymComma ||
                self.current_token().token_inner == TokenInner::SymRParen
            {
                if has_colon {
                    self.diag.borrow_mut()
                        .diag(self.current_token().range.left(), diag_data::err_missing_type_got_0)
                        .add_arg2(self.current_token().token_inner)
                        .add_mark(self.current_token().range.into())
                        .emit();
                    self.skip_to_any_of(failsafe_set);
                    return None;
                }
                None
            } else {
                Some(self.parse_type(failsafe_set)?)
            };
        Some(FunctionParam {
            attr,
            param_name,
            param_type
        })
    }

    pub fn parse_func_ret_type(&mut self, failsafe_set: &[&[TokenInner<'_>]])
        -> Option<Vec<ConcreteType<'s>>>
    {
        if self.current_token().token_inner == TokenInner::SymLParen {
            let _ = self.consume_token();
            let (types, _): (Vec<ConcreteType<'s>>, _) = self.parse_list_alike_nonnull(
                Self::parse_type,
                failsafe_set,
                TokenInner::SymComma,
                TokenInner::SymRParen,
                failsafe_set
            )?;
            Some(types)
        } else {
            let ty: ConcreteType = self.parse_type(failsafe_set)?;
            Some(vec![ty])
        }
    }

    pub fn parse_export_decl(&mut self, kwd_token: Token<'s>, failsafe_set: &[&[TokenInner<'_>]])
        -> Option<ConcreteExportDecl<'s>>
    {
        let left_paren_loc: SourceLoc =
            self.expect_n_consume(TokenInner::SymLParen, failsafe_set)?.range.left();
        let (idents, right_paren_range): (Vec<Identifier<'s>>, SourceRange) =
            self.parse_list_alike_nonnull(
                Self::parse_ident_with_skip,
                failsafe_set,
                TokenInner::SymComma,
                TokenInner::SymRParen,
                failsafe_set
            )?;
        self.expect_n_consume(TokenInner::SymSemicolon, failsafe_set)?;
        let right_paren_loc: SourceLoc = right_paren_range.left();

        Some(ConcreteExportDecl {
            exported_idents: idents,
            export_kwd_range: kwd_token.range,
            left_paren_loc,
            right_paren_loc
        })
    }

    pub fn parse_import_decl(&mut self, kwd_token: Token<'s>, failsafe_set: &[&[TokenInner<'_>]])
                             -> Option<ConcreteImportDecl<'s>>
    {
        let import_path: Identifier<'s> = self.parse_ident_with_skip(failsafe_set)?;
        self.expect_n_consume(TokenInner::SymSemicolon, failsafe_set)?;
        Some(ConcreteImportDecl {
            import_path,
            import_kwd_range: kwd_token.range,

            is_syntax_action: false
        })
    }

    pub fn parse_open_import_decl(
        &mut self,
        kwd_token: Token<'s>,
        failsafe_set: &[&[TokenInner<'_>]]
    ) -> Option<ConcreteOpenImportDecl<'s>> {
        let import_kwd_range: SourceRange =
            self.expect_n_consume(TokenInner::KwdImport, failsafe_set)?.range;
        let import_path: Identifier<'s> = self.parse_ident_with_skip(failsafe_set)?;
        let using_kwd_range: SourceRange =
            self.expect_n_consume(TokenInner::KwdUsing, failsafe_set)?.range;
        let left_paren_loc: SourceLoc =
            self.expect_n_consume(TokenInner::SymLParen, failsafe_set)?.range.left();
        let (use_item_list, right_paren_range): (Vec<OpenImportUsingItem<'s>>, SourceRange) =
            self.parse_list_alike_nonnull(
                Self::parse_open_import_using_item,
                failsafe_set,
                TokenInner::SymComma,
                TokenInner::SymRParen,
                failsafe_set
            )?;
        let right_paren_loc: SourceLoc = right_paren_range.left();

        Some(ConcreteOpenImportDecl {
            import_path,
            use_item_list,
            open_kwd_range: kwd_token.range,
            import_kwd_range,
            using_kwd_range,
            using_list_left_paren_loc: left_paren_loc,
            using_list_right_paren_loc: right_paren_loc
        })
    }

    pub fn parse_open_import_using_item(
        &mut self,
        failsafe_set: &[&[TokenInner<'_>]]
    ) -> Option<OpenImportUsingItem<'s>> {
        match self.current_token().token_inner {
            TokenInner::Ident(_) => {
                let ident: Identifier<'s> = self.parse_ident_with_skip(failsafe_set)?;
                if self.current_token().token_inner == TokenInner::KwdAs {
                    self.consume_token();
                    let as_ident: Identifier<'s> = self.parse_unqual_ident_with_skip(failsafe_set)?;
                    Some(OpenImportUsingItem::UsingIdent {
                        ident, as_ident: Some(as_ident), is_syntax_action: false
                    })
                } else {
                    Some(OpenImportUsingItem::UsingIdent {
                        ident, as_ident: None, is_syntax_action: false
                    })
                }
            },
            TokenInner::SymAster => {
                let aster_loc: SourceLoc = self.consume_token().range.left();
                Some(OpenImportUsingItem::UsingAny { aster_loc })
            },
            _ => {
                let current_token_range: SourceRange = self.current_token().range;
                self.diag.borrow_mut()
                    .diag(current_token_range.left(), diag_data::err_expected_any_of_0_got_1)
                    .add_arg2(awa![TokenInner::Ident(""), TokenInner::SymAster])
                    .add_arg2(self.current_token().token_inner)
                    .add_mark(current_token_range.into())
                    .emit();
                self.skip_to_any_of(failsafe_set);
                None
            }
        }
    }
}

#[cfg(test)]
mod test {
    use std::cell::RefCell;

    use crate::diag::DiagContext;
    use crate::parse::parser::Parser;
    use crate::syntax::decl::{ConcreteExportDecl, ConcreteFuncDecl, ConcreteImportDecl, ConcreteObjectDecl, ConcreteOpenImportDecl};
    use crate::syntax::token::Token;

    #[test]
    fn test_parse_object_decl() {
        let source: &str = "const a = b::c::d.e().await;";

        let diag: RefCell<DiagContext> = RefCell::new(DiagContext::new());
        let mut parser: Parser = Parser::new(
            0, source, &diag
        );

        let kwd_token: Token = parser.consume_token();
        let decl: ConcreteObjectDecl = parser.parse_object_decl(kwd_token, &[]).unwrap();

        dbg!(decl);
    }

    #[test]
    fn test_parse_object_decl2() {
        let source: &str = "const a vector<int> = b::c::d.e().await;";

        let diag: RefCell<DiagContext> = RefCell::new(DiagContext::new());
        let mut parser: Parser = Parser::new(
            0, source, &diag
        );

        let kwd_token: Token = parser.consume_token();
        let decl: ConcreteObjectDecl = parser.parse_object_decl(kwd_token, &[]).unwrap();

        dbg!(decl);
    }

    #[test]
    fn test_parse_object_decl3() {
        let source: &str = "const a: vector<string> = b::c::d.e(f, g).await;";

        let diag: RefCell<DiagContext> = RefCell::new(DiagContext::new());
        let mut parser: Parser = Parser::new(
            0, source, &diag
        );

        let kwd_token: Token = parser.consume_token();
        let decl: ConcreteObjectDecl = parser.parse_object_decl(kwd_token, &[]).unwrap();

        dbg!(decl);
    }

    #[test]
    fn test_parse_func() {
        let source: &str = "fn foo(bar int, #[reflect] baz: vector<string>) string;";

        let diag: RefCell<DiagContext> = RefCell::new(DiagContext::new());
        let mut parser: Parser = Parser::new(
            0, source, &diag
        );

        let kwd_token: Token = parser.consume_token();
        let func: ConcreteFuncDecl = parser.parse_func_decl(kwd_token, &[]).unwrap();

        dbg!(func);
    }

    #[test]
    fn test_parse_export() {
        let source: &str = "export (foo, bar::baz);";

        let diag: RefCell<DiagContext> = RefCell::new(DiagContext::new());
        let mut parser: Parser = Parser::new(
            0, source, &diag
        );

        let kwd_token: Token = parser.consume_token();
        let export: ConcreteExportDecl = parser.parse_export_decl(kwd_token, &[]).unwrap();

        dbg!(export);
    }

    #[test]
    fn test_parse_import() {
        let source: &str = "import foo::bar::baz;";

        let diag: RefCell<DiagContext> = RefCell::new(DiagContext::new());
        let mut parser: Parser = Parser::new(
            0, source, &diag
        );
        let kwd_token: Token = parser.consume_token();
        let import: ConcreteImportDecl = parser.parse_import_decl(kwd_token, &[]).unwrap();

        dbg!(import);
    }

    #[test]
    fn test_parse_open_import() {
        let source: &str = "open import foo::bar::baz using (f, g as h);";

        let diag: RefCell<DiagContext> = RefCell::new(DiagContext::new());
        let mut parser: Parser = Parser::new(
            0, source, &diag
        );
        let kwd_token: Token = parser.consume_token();
        let import: ConcreteOpenImportDecl = parser.parse_open_import_decl(kwd_token, &[]).unwrap();

        dbg!(import);
    }

    #[test]
    fn test_parse_open_import2() {
        let source: &str = "open import foo::bar::baz using (*, a, b::c, d::e::f as nothing);";

        let diag: RefCell<DiagContext> = RefCell::new(DiagContext::new());
        let mut parser: Parser = Parser::new(
            0, source, &diag
        );
        let kwd_token: Token = parser.consume_token();
        let import: ConcreteOpenImportDecl = parser.parse_open_import_decl(kwd_token, &[]).unwrap();

        dbg!(import);
    }
}
