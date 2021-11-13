pub mod attr;
pub mod base;
pub mod decl;
pub mod top_level;

use std::cell::RefCell;
use std::mem::swap;

use unchecked_unwrap::UncheckedUnwrap;

use crate::diag::{DiagContext, DiagMark};
use crate::diag::diag_data;
use crate::parse::lexer::Lexer;
use crate::syntax::token::{Token, TokenInner};

pub struct Parser<'src, 'diag> {
    lexer: Lexer<'src, 'diag>,
    m_current_token: Token<'src>,
    m_peek_token: Option<Token<'src>>,

    #[allow(unused)] file_id: u32,
    #[allow(unused)] source: &'src str,
    diag: &'diag RefCell<DiagContext>
}

impl<'s, 'd> Parser<'s, 'd> {
    pub fn new(file_id: u32, source: &'s str, diag: &'d RefCell<DiagContext>) -> Self {
        let mut lexer: Lexer<'s, 'd> = Lexer::new(file_id, source, diag);
        let current_token: Token<'s> = lexer.next_token();
        let peek_token: Option<Token<'s>> = None;

        Parser {
            lexer,
            m_current_token: current_token,
            m_peek_token: peek_token,

            file_id,
            source,
            diag
        }
    }

    fn consume_token(&mut self) -> Token<'s> {
        if self.m_current_token.is_eoi() {
            return Token::new_eoi(self.m_current_token.range);
        }

        let mut token: Token<'s> = if let Some(peek_token) = self.m_peek_token.take() {
            peek_token
        } else {
            self.lexer.next_token()
        };
        swap(&mut token, &mut self.m_current_token);
        token
    }

    fn current_token(&self) -> &Token<'s> {
        &self.m_current_token
    }

    fn peek_token(&mut self) -> &Token<'s> {
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
        token_kind: TokenInner<'_>,
        skip_on_failure: &[&[TokenInner<'_>]]
    ) -> Option<&Token<'s>> {
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
        token_kind: TokenInner<'_>,
        skip_on_failure: &[&[TokenInner<'_>]]
    ) -> Option<Token<'s>> {
        if let Some(_) = self.expect_token(token_kind, skip_on_failure) {
            Some(self.consume_token())
        } else {
            None
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

    #[allow(unused)]
    fn skip_when(&mut self, skipped: &[&[TokenInner<'_>]]) {
        while !self.current_token().is_eoi() {
            if skipped.iter().any(|tokens| tokens.contains(&self.current_token().token_inner)) {
                self.consume_token();
            } else {
                break;
            }
        }
    }

    fn skip_to_any_of(&mut self, skip_choices: &[&[TokenInner<'_>]]) {
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
    TOP_LEVEL_DECL_FIRST
];
