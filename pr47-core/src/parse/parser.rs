pub mod attr;
pub mod base;
pub mod decl;
pub mod expr;
pub mod stmt;
pub mod top_level;
pub mod ty;

use std::cell::RefCell;
use std::mem::swap;

use unchecked_unwrap::UncheckedUnwrap;
use xjbutil::defer;

use crate::diag::{DiagContext, DiagMark};
use crate::diag::diag_data;
use crate::parse::lexer::{Lexer, LexerMode};
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

    #[allow(unused)]
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
                .add_arg2(token_kind)
                .add_arg2(self.current_token().token_inner)
                .add_mark(
                    DiagMark::from(self.current_token().range).add_comment("unexpected token")
                )
                .emit();
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
        self.without_attr_mode(|this: &mut Self| {
            while !this.current_token().is_eoi() {
                if skipped.iter().any(|tokens| tokens.contains(&this.current_token().token_inner)) {
                    this.consume_token();
                } else {
                    break;
                }
            }
        });
    }

    fn skip_to_any_of(&mut self, skip_choices: &[&[TokenInner<'_>]]) {
        self.without_attr_mode(|this: &mut Self| {
            while !this.current_token().is_eoi() {
                for skip_choice in skip_choices {
                    if skip_choice.contains(&this.current_token().token_inner) {
                        return;
                    }
                }
                this.consume_token();
            }
        });
    }

    fn without_attr_mode<T>(&mut self, skip_operation: impl FnOnce(&mut Self) -> T) -> T {
        if self.lexer.current_mode() == LexerMode::LexAttr {
            let this: &mut Self = self;
            this.lexer.push_lexer_mode(this.lexer.prev_mode());
            defer!(|this: &mut Self| {
                this.lexer.pop_lexer_mode();
            }, this);
            skip_operation(this)
        } else {
            skip_operation(self)
        }
    }

    #[cfg(test)]
    pub fn push_lexer_mode(&mut self, lexer_mode: LexerMode) {
        self.lexer.push_lexer_mode(lexer_mode);
    }

    #[cfg(test)]
    pub fn pop_lexer_mode(&mut self) {
        self.lexer.pop_lexer_mode();
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
