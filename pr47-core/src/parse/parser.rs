use std::cell::RefCell;

use crate::diag::DiagContext;
use crate::syntax::token::Token;
use crate::parse::lexer::Lexer;

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
        let mut lexer = Lexer::new(file_id, source, diag);
        let current_token = lexer.next_token();
        let peek_token = None;

        Parser {
            lexer,
            current_token,
            peek_token,

            file_id,
            source,
            diag
        }
    }
}
