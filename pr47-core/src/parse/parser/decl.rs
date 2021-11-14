use super::Parser;

use crate::syntax::decl::ConcreteObjectDecl;
use crate::syntax::token::{Token, TokenInner};

impl<'s, 'd> Parser<'s, 'd> {
    pub fn parse_object_decl(&mut self, kwd_token: Token<'s>, _failsafe_set: &[&[TokenInner<'_>]])
        -> Option<ConcreteObjectDecl<'s>>
    {
        let _kwd_range = kwd_token.range;

        todo!()
    }
}
