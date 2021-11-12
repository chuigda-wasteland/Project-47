use super::Parser;
use super::{TOP_LEVEL_DECL_FAILSAFE};

use crate::diag::diag_data;
use crate::diag::location::SourceRange;
use crate::syntax::decl::{ConcreteDecl, ConcreteObjectDecl};
use crate::syntax::token::TokenInner;

impl<'a, 'b> Parser<'a, 'b> {
    pub fn parse_top_level_decl(&mut self) -> Option<ConcreteDecl<'a>> {
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
        -> Option<ConcreteObjectDecl<'a>>
    {
        let _kwd_range: SourceRange = self.current_token().range;
        self.consume_token();

        todo!()
    }
}
