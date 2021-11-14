use super::Parser;

use smallvec::{SmallVec, smallvec};
use xjbutil::defer;

use crate::diag::location::SourceRange;
use crate::parse::lexer::LexerMode;
use crate::syntax::token::{Token, TokenInner};
use crate::syntax::ty::ConcreteType;

impl<'s, 'd> Parser<'s, 'd> {
    pub fn parse_type(&mut self, failsafe_set: &[&[TokenInner<'_>]]) -> Option<ConcreteType<'s>> {
        let this: &mut Parser<'s, 'd> = self;

        defer!(|this: &mut Parser<'s, 'd>| {
            this.lexer.pop_lexer_mode()
        }, this);

        this.lexer.push_lexer_mode(LexerMode::LexType);
        this.parse_type_impl(failsafe_set)
    }

    fn parse_type_impl(&mut self, failsafe_set: &[&[TokenInner<'_>]]) -> Option<ConcreteType<'s>> {
        match self.current_token().token_inner {
            TokenInner::KwdAny | TokenInner::KwdChar | TokenInner::KwdFloat |
            TokenInner::KwdInt | TokenInner::KwdObject | TokenInner::KwdString |
            TokenInner::KwdVoid => {
                Some(ConcreteType::PrimitiveType(self.consume_token()))
            },
            TokenInner::KwdVector => {
                let container_type_token: Token<'s> = self.consume_token();
                self.parse_generic_type(container_type_token, failsafe_set)
            }
            _ => todo!()
        }
    }

    pub fn parse_generic_type(
        &mut self,
        container_type_token: Token<'s>,
        failsafe_set: &[&[TokenInner<'_>]]
    ) -> Option<ConcreteType<'s>> {
        debug_assert_eq!(container_type_token.token_inner, TokenInner::KwdVector);

        let _left_angle_range: SourceRange =
            self.expect_n_consume(TokenInner::SymLt, failsafe_set)?.range;

        let first_type_param: ConcreteType = self.parse_type(failsafe_set)?;
        let _type_parm_list: SmallVec<[ConcreteType<'s>; 2]> = smallvec![first_type_param];

        todo!()
    }
}
