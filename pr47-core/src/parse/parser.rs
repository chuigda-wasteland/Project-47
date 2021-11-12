pub mod attr;
pub mod decl;

use std::cell::RefCell;
use std::mem::swap;

use unchecked_unwrap::UncheckedUnwrap;
use xjbutil::either::Either;

use crate::diag::{DiagContext, DiagMark};
use crate::diag::diag_data;
use crate::parse::lexer::Lexer;
use crate::syntax::ConcreteProgram;
use crate::syntax::attr::Attribute;
use crate::syntax::decl::ConcreteDecl;
use crate::syntax::token::{Token, TokenInner};
use crate::diag::location::SourceRange;

pub struct Parser<'a, 'b> {
    lexer: Lexer<'a, 'b>,
    m_current_token: Token<'a>,
    m_peek_token: Option<Token<'a>>,

    #[allow(unused)] file_id: u32,
    #[allow(unused)] source: &'a str,
    diag: &'b RefCell<DiagContext>
}

impl<'a, 'b> Parser<'a, 'b> {
    pub fn new(file_id: u32, source: &'a str, diag: &'b RefCell<DiagContext>) -> Self {
        let mut lexer: Lexer<'a, 'b> = Lexer::new(file_id, source, diag);
        let current_token: Token<'a> = lexer.next_token();
        let peek_token: Option<Token<'a>> = None;

        Parser {
            lexer,
            m_current_token: current_token,
            m_peek_token: peek_token,

            file_id,
            source,
            diag
        }
    }

    fn consume_token(&mut self) -> Token<'a> {
        if self.m_current_token.is_eoi() {
            return Token::new_eoi(self.m_current_token.range);
        }

        let mut token: Token<'a> = if let Some(peek_token) = self.m_peek_token.take() {
            peek_token
        } else {
            self.lexer.next_token()
        };
        swap(&mut token, &mut self.m_current_token);
        token
    }

    fn current_token(&self) -> &Token<'a> {
        &self.m_current_token
    }

    fn peek_token(&mut self) -> &Token<'a> {
        if self.m_current_token.is_eoi() {
            return &self.m_current_token;
        }

        if self.m_peek_token.is_none() {
            self.m_peek_token = Some(self.lexer.next_token())
        }
        unsafe { self.m_peek_token.as_ref().unchecked_unwrap() }
    }

    #[must_use = "not using the return value is likely to be a mistake"]
    pub fn expect_token(
        &mut self,
        token_kind: TokenInner<'static>,
        skip_on_failure: &[&[TokenInner<'static>]]
    ) -> Option<&Token<'a>> {
        if self.current_token().token_inner != token_kind {
            self.diag.borrow_mut()
                .diag(self.current_token().range.left(), diag_data::err_expected_token_0_got_1)
                // TODO: diagPrettyPrint : TokenInner -> String
                .add_arg("todo")
                .add_arg("todo")
                .add_mark(
                    DiagMark::from(self.current_token().range).add_comment("unexpected token")
                )
                .build();
            if skip_on_failure.len() != 0 {
                self.skip_to_any_of(skip_on_failure);
            }
            None
        } else {
            Some(self.current_token())
        }
    }

    #[must_use = "not using the return value is likely to be a mistake"]
    pub fn expect_n_consume(
        &mut self,
        token_kind: TokenInner<'static>,
        skip_on_failure: &[&[TokenInner<'static>]]
    ) -> Option<Token<'a>> {
        if self.current_token().token_inner != token_kind {
            self.diag.borrow_mut()
                .diag(self.current_token().range.left(), diag_data::err_expected_token_0_got_1)
                // TODO: diagPrettyPrint : TokenInner -> String
                .add_arg("todo")
                .add_arg("todo")
                .add_mark(
                    DiagMark::from(self.current_token().range).add_comment("unexpected token")
                )
                .build();
            if skip_on_failure.len() != 0 {
                self.skip_to_any_of(skip_on_failure);
            }
            None
        } else {
            Some(self.consume_token())
        }
    }

    #[allow(unused)]
    fn skip_optional(&mut self, token_kind: TokenInner<'static>) -> bool {
        if self.current_token().token_inner != token_kind {
            false
        } else {
            self.consume_token();
            true
        }
    }

    fn skip_to_any_of(&mut self, skip_choices: &[&[TokenInner<'static>]]) {
        while !self.current_token().is_eoi() {
            for skip_choice in skip_choices {
                if skip_choice.contains(&self.current_token().token_inner) {
                    return;
                }
            }
            self.consume_token();
        }
    }
}

const TOP_LEVEL_DECL_FIRST: &'static [TokenInner<'static>] = &[
    TokenInner::KwdConst,
    TokenInner::KwdExport,
    TokenInner::KwdFunc,
    TokenInner::KwdImport,
    TokenInner::KwdOpen,
];

const ATTR_FIRST: &'static [TokenInner<'static>] = &[TokenInner::SymHash];

const TOP_LEVEL_FIRST: &'static [&'static [TokenInner<'static>]] = &[
    TOP_LEVEL_DECL_FIRST,
    ATTR_FIRST
];

const TOP_LEVEL_DECL_FAILSAFE: &'static [&'static [TokenInner<'static>]] = &[
    TOP_LEVEL_DECL_FIRST,
];

impl<'a, 'b> Parser<'a, 'b> {
    pub fn parse(&mut self) -> ConcreteProgram<'a> {
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
        -> Option<Either<ConcreteDecl<'a>, Attribute<'a>>>
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

    pub fn parse_attributed_top_level_decl(&mut self) -> Option<ConcreteDecl<'a>> {
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

    pub fn diag_unexpected_eoi(&mut self, range: SourceRange) {
        self.diag.borrow_mut()
            .diag(range.left(), diag_data::err_unexpected_eoi)
            .build()
    }
}
