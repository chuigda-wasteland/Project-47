use std::cell::RefCell;

use smallvec::{SmallVec, smallvec};
use unchecked_unwrap::UncheckedUnwrap;
use xjbutil::either::Either;

use crate::diag::{DiagContext, DiagMark};
use crate::diag::diag_data;
use crate::parse::lexer::Lexer;
use crate::syntax::ConcreteProgram;
use crate::syntax::attr::{AttrList, Attribute};
use crate::syntax::decl::{ConcreteDecl, ConcreteObjectDecl};
use crate::syntax::token::{Token, TokenInner};
use crate::diag::location::SourceRange;

#[allow(dead_code)]
pub struct Parser<'a, 'b> {
    lexer: Lexer<'a, 'b>,
    m_current_token: Token<'a>,
    m_peek_token: Option<Token<'a>>,

    file_id: u32,
    source: &'a str,
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

    fn consume_token(&mut self) {
        if self.m_current_token.is_eoi() {
            return;
        }

        if let Some(peek_token) = self.m_peek_token.take() {
            self.m_current_token = peek_token;
        } else {
            self.m_current_token = self.lexer.next_token();
        }
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

    fn expect_token(
        &mut self,
        token_kind: TokenInner<'static>,
        skip_on_failure: &[&[TokenInner<'static>]]
    ) -> Option<&Token<'a>> {
        if self.current_token().token_inner != token_kind {
            self.diag.borrow_mut()
                .diag(self.current_token().range.left(), diag_data::err_expected_token_0_got_1)
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
    pub fn parse(&mut self) -> ConcreteProgram {
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
        -> Option<Either<ConcreteDecl, AttrList>>
    {
        match self.peek_token().token_inner {
            TokenInner::SymLBracket => {
                self.parse_attributed_top_level_decl().map(|attributed_decl| Either::Left(attributed_decl))
            },
            TokenInner::SymExclaim => {
                self.parse_attr_list(true).map(|global_attr| Either::Right(global_attr))
            },
            _ => {
                todo!("error reporting")
            }
        }
    }

    pub fn parse_attributed_top_level_decl(&mut self) -> Option<ConcreteDecl> {
        let attr_list: AttrList = self.parse_attr_list(false)?;
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

    pub fn parse_attr_list(&mut self, is_global: bool) -> Option<AttrList> {
        let hash_range: SourceRange = self.current_token().range;
        self.consume_token();

        let exclaim_range = if is_global {
            let exclaim_range: SourceRange =
                self.expect_token(TokenInner::SymExclaim, TOP_LEVEL_FIRST)
                    .expect("only appoint `is_global` when here's surely a global attribute")
                    .range;
            self.consume_token();
            exclaim_range
        } else {
            SourceRange::unknown()
        };

        let left_bracket_range: SourceRange =
            self.expect_token(TokenInner::SymLBracket, TOP_LEVEL_FIRST)?.range;
        self.consume_token();

        let mut attributes: SmallVec<[Attribute; 4]> = smallvec![];
        let right_bracket_range: SourceRange = loop {
            if self.current_token().is_eoi() {
                self.diag_unexpected_eoi(self.current_token().range);
                return None;
            }

            if let Some(attr) = self.parse_attribute() {
                attributes.push(attr);
                if self.m_current_token.token_inner == TokenInner::SymComma {
                    self.consume_token();
                } else {
                    let right_bracket_range: SourceRange =
                        self.expect_token(TokenInner::SymRBracket, &[])?.range;
                    self.consume_token();
                    break right_bracket_range;
                }
            } else {
                return None;
            }
        };

        Some(AttrList {
            attributes,

            hash_loc: hash_range.left(),
            exclaim_loc: exclaim_range.left(),
            left_bracket_loc: left_bracket_range.left(),
            right_bracket_loc: right_bracket_range.left()
        })
    }

    pub fn parse_top_level_decl(&mut self) -> Option<ConcreteDecl> {
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

    pub fn parse_object_decl(&mut self, _failsafe_set: &[&[TokenInner]])
        -> Option<ConcreteObjectDecl>
    {
        let _kwd_range: SourceRange = self.current_token().range;
        self.consume_token();

        todo!()
    }

    pub fn parse_attribute(&mut self) -> Option<Attribute> {
        todo!()
    }

    pub fn diag_unexpected_eoi(&mut self, range: SourceRange) {
        self.diag.borrow_mut()
            .diag(range.left(), diag_data::err_unexpected_eoi)
            .build()
    }
}
