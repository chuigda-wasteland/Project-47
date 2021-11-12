use super::Parser;

use smallvec::{SmallVec, smallvec};

use crate::syntax::id::Identifier;
use crate::syntax::token::{Token, TokenInner};

impl<'s, 'd> Parser<'s, 'd> {
    pub fn parse_ident(&mut self) -> Option<Identifier<'s>> {
        let token: Token<'s> = self.expect_n_consume(TokenInner::Ident(""), &[])?;
        if self.current_token().token_inner != TokenInner::SymDColon {
            return Some(Identifier::Unqual(token))
        }

        let mut token_buffer: SmallVec<[Token<'s>; 2]> = smallvec![token];
        while self.current_token().token_inner == TokenInner::SymDColon {
            self.consume_token();
            let token: Token<'s> = self.expect_n_consume(TokenInner::Ident(""), &[])?;
            token_buffer.push(token);
        }

        Some(Identifier::Qual(token_buffer))
    }

    pub fn parse_unqual_ident(&mut self) -> Option<Identifier<'s>> {
        let token: Token<'s> = self.expect_n_consume(TokenInner::Ident(""), &[])?;
        if self.current_token().token_inner == TokenInner::SymDColon {
            todo!("report error and recovery")
        }

        Some(Identifier::Unqual(token))
    }
}
