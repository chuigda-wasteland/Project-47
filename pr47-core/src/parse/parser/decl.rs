use super::Parser;

use crate::diag::location::SourceRange;
use crate::syntax::decl::ConcreteObjectDecl;
use crate::syntax::token::TokenInner;

impl<'s, 'd> Parser<'s, 'd> {
    pub fn parse_object_decl(&mut self, _failsafe_set: &[&[TokenInner]])
        -> Option<ConcreteObjectDecl<'s>>
    {
        let _kwd_range: SourceRange = self.current_token().range;
        self.consume_token();

        todo!()
    }
}
