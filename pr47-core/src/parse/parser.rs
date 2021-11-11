use std::cell::RefCell;

use smallvec::{SmallVec, smallvec};
use unchecked_unwrap::UncheckedUnwrap;

use crate::diag::{DiagContext, DiagMark};
use crate::diag::diag_data;
use crate::parse::lexer::Lexer;
use crate::syntax::token::{Token, TokenInner};
use crate::syntax::ConcreteProgram;
use crate::syntax::attr::{AttrList, Attribute};
use crate::syntax::decl::ConcreteDecl;
use crate::diag::location::SourceRange;

#[allow(dead_code)]
pub struct Parser<'a, 'b> {
    lexer: Lexer<'a, 'b>,
    current_token: Token<'a>,
    peek_token: Option<Token<'a>>,

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
            current_token,
            peek_token,

            file_id,
            source,
            diag
        }
    }

    fn consume_token(&mut self) {
        if self.current_token.is_eoi() {
            return;
        }

        if let Some(peek_token) = self.peek_token.take() {
            self.current_token = peek_token;
        } else {
            self.current_token = self.lexer.next_token();
        }
    }

    fn current_token(&self) -> &Token<'a> {
        &self.current_token
    }

    #[allow(unused)]
    fn peek_token(&mut self) -> &Token<'a> {
        if self.current_token.is_eoi() {
            return &self.current_token;
        }

        if self.peek_token.is_none() {
            self.peek_token = Some(self.lexer.next_token())
        }
        unsafe { self.peek_token.as_ref().unchecked_unwrap() }
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

const TOP_LEVEL_DECL_COMMENCE: &'static [TokenInner<'static>] = &[
    TokenInner::KwdConst,
    TokenInner::KwdExport,
    TokenInner::KwdFunc,
    TokenInner::KwdImport,
    TokenInner::KwdOpen,
];

impl<'a, 'b> Parser<'a, 'b> {
    pub fn parse(&mut self) -> ConcreteProgram {
        let mut program = ConcreteProgram::new();
        while !self.current_token.is_eoi() {
            if self.current_token.token_inner == TokenInner::SymHash {
                if let Some(global_attr) = self.parse_global_attr() {
                    program.global_attrs.push(global_attr);
                }
            } else {
                if let Some(top_level_decl) = self.parse_top_level_decl() {
                    program.decls.push(top_level_decl);
                }
            }
        }

        program
    }

    pub fn parse_global_attr(&mut self) -> Option<AttrList> {
        let sharp_range: SourceRange = self.current_token().range;

        self.consume_token();
        let exclaim_range: SourceRange =
            self.expect_token(TokenInner::SymExclaim, &[TOP_LEVEL_DECL_COMMENCE])?.range;

        self.consume_token();
        let left_bracket_range: SourceRange =
            self.expect_token(TokenInner::SymLBracket, &[TOP_LEVEL_DECL_COMMENCE])?.range;

        let mut attributes: SmallVec<[Attribute; 4]> = smallvec![];
        self.consume_token();
        let right_bracket_range: SourceRange = loop {
            if self.current_token().is_eoi() {
                self.diag_unexpected_eoi(self.current_token().range);
                return None;
            }

            if let Some(attr) = self.parse_attribute() {
                attributes.push(attr);
                if self.current_token.token_inner == TokenInner::SymComma {
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

            sharp_loc: sharp_range.left(),
            exclaim_loc: exclaim_range.left(),
            left_bracket_loc: left_bracket_range.left(),
            right_bracket_loc: right_bracket_range.left()
        })
    }

    pub fn parse_top_level_decl(&mut self) -> Option<ConcreteDecl> {
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
