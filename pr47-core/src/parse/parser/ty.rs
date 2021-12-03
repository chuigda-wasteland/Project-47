use super::Parser;

use smallvec::SmallVec;
use xjbutil::defer;

use crate::awa;
use crate::diag::diag_data;
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
            TokenInner::KwdAny | TokenInner::KwdBool | TokenInner::KwdChar |
            TokenInner::KwdFloat | TokenInner::KwdInt | TokenInner::KwdObject |
            TokenInner::KwdString | TokenInner::KwdVoid => {
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
            _ => {
                self.diag.borrow_mut()
                    .diag(self.current_token().range.left(), diag_data::err_expected_any_of_0_got_1)
                    .add_arg2(awa![
                        TokenInner::KwdAny,
                        TokenInner::KwdBool,
                        TokenInner::KwdChar,
                        TokenInner::KwdFloat,
                        TokenInner::KwdInt,
                        TokenInner::KwdObject,
                        TokenInner::KwdString,
                        TokenInner::KwdVoid,
                        TokenInner::KwdVector,
                        TokenInner::KwdAuto,
                        TokenInner::Ident("")
                    ])
                    .add_arg2(self.current_token().token_inner)
                    .add_mark(self.current_token().range.into())
                    .emit();
                self.skip_to_any_of(failsafe_set);
                None
            }
        }
    }

    pub fn parse_generic_type(
        &mut self,
        container_type_token: Token<'s>,
        failsafe_set: &[&[TokenInner<'_>]]
    ) -> Option<ConcreteType<'s>> {
        #[cfg(debug_assertions)]
        assert_eq!(container_type_token.token_inner, TokenInner::KwdVector);

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

#[cfg(test)]
mod test {
    use std::cell::RefCell;

    use crate::diag::DiagContext;
    use crate::parse::parser::Parser;
    use crate::syntax::id::assert_ident_qual;
    use crate::syntax::token::TokenInner;
    use crate::syntax::ty::ConcreteType;

    fn test_primitive_type(ty: &ConcreteType, token_inner: TokenInner) {
        if let ConcreteType::PrimitiveType(token) = ty {
            assert_eq!(token.token_inner, token_inner);
        } else {
            panic!("shoud be a primitive type")
        }
    }

    #[test]
    fn test_parse_primitive_type() {
        let source: &str = "any bool char float int object string void";
        let diag: RefCell<DiagContext> = RefCell::new(DiagContext::new());
        let mut parser: Parser = Parser::new(
            0, source, &diag
        );

        {
            use TokenInner::*;
            for token_inner in [
                KwdAny, KwdBool, KwdChar, KwdFloat, KwdInt, KwdObject, KwdString, KwdVoid
            ] {
                let ty: ConcreteType = parser.parse_type(&[]).unwrap();
                test_primitive_type(&ty, token_inner);
            }
        }

        assert!(!diag.borrow().has_error());
    }

    #[test]
    fn test_parse_deduced_type() {
        let source: &str = "auto";
        let diag: RefCell<DiagContext> = RefCell::new(DiagContext::new());
        let mut parser: Parser = Parser::new(
            0, source, &diag
        );

        if let ConcreteType::DeducedType(_) = parser.parse_type(&[]).unwrap() {} else {
            panic!("should be a deduced type")
        }
    }

    #[test]
    fn test_parse_generic_type() {
        let source: &str = "vector<vector<string>, std::char_traits, int>";
        let diag: RefCell<DiagContext> = RefCell::new(DiagContext::new());
        let mut parser: Parser = Parser::new(
            0, source, &diag
        );

        let ty: ConcreteType = parser.parse_type(&[]).unwrap();
        if let ConcreteType::GenericType(generic_type) = ty {
            assert_eq!(generic_type.base.token_inner, TokenInner::KwdVector);
            assert_eq!(generic_type.inner.len(), 3);

            if let ConcreteType::GenericType(generic_type) = &generic_type.inner[0] {
                assert_eq!(generic_type.base.token_inner, TokenInner::KwdVector);
                assert_eq!(generic_type.inner.len(), 1);

                test_primitive_type(&generic_type.inner[0], TokenInner::KwdString);
            } else {
                panic!("should be a generic type");
            }

            if let ConcreteType::UserType(ident) = &generic_type.inner[1] {
                assert_ident_qual(&ident, &["std", "char_traits"]);
            } else {
                panic!("should be a user type");
            }

            test_primitive_type(&generic_type.inner[2], TokenInner::KwdInt);
        } else {
            panic!("should be a generic type");
        }
    }
}
