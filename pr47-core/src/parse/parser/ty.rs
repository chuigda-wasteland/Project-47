use super::Parser;

use smallvec::SmallVec;
use xjbutil::defer;

use crate::diag::location::SourceRange;
use crate::parse::lexer::LexerMode;
use crate::syntax::id::Identifier;
use crate::syntax::token::{Token, TokenInner};
use crate::syntax::ty::{ConcreteGenericType, ConcreteType};

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
            },
            TokenInner::KwdAuto => {
                Some(ConcreteType::DeducedType(self.consume_token().range))
            },
            TokenInner::Ident(_) => {
                let ident: Identifier<'s> = self.parse_ident().or_else(|| {
                    self.skip_to_any_of(failsafe_set);
                    None
                })?;
                Some(ConcreteType::UserType(ident))
            },
            _ => todo!()
        }
    }

    pub fn parse_generic_type(
        &mut self,
        container_type_token: Token<'s>,
        failsafe_set: &[&[TokenInner<'_>]]
    ) -> Option<ConcreteType<'s>> {
        debug_assert_eq!(container_type_token.token_inner, TokenInner::KwdVector);

        let left_angle_range: SourceRange =
            self.expect_n_consume(TokenInner::SymLt, failsafe_set)?.range;
        let (type_param_list, right_angle_range): (SmallVec<[ConcreteType<'s>; 2]>, SourceRange) =
            self.parse_list_alike_nonnull(
                Self::parse_type,
                failsafe_set,
                TokenInner::SymComma,
                TokenInner::SymGt,
                failsafe_set
            )?;

        Some(ConcreteType::GenericType(Box::new(
                ConcreteGenericType {
                base: container_type_token,
                inner: type_param_list,
                left_angle: left_angle_range.left(),
                right_angle: right_angle_range.right()
            }
        )))
    }
}
