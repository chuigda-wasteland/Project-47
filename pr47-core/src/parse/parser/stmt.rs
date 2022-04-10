use super::Parser;

use crate::syntax::stmt::ConcreteCompoundStmt;
use crate::syntax::token::{Token, TokenInner};

impl<'s, 'd> Parser<'s, 'd> {
    pub fn parse_compound_stmt(
        &mut self,
        _lbrace_token: Token<'s>,
        _failsafe_set: &[&[TokenInner<'_>]]
    ) -> Option<ConcreteCompoundStmt<'s>> {
        todo!();
    }
}
