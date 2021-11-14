use super::Parser;

use smallvec::SmallVec;
use xjbutil::defer;

use crate::diag::diag_data;
use crate::diag::location::SourceRange;
use crate::parse::lexer::LexerMode;
use crate::syntax::attr::{AttrAssignLikeItem, AttrCallLikeItem, AttrItem, AttrValue, Attribute};
use crate::syntax::id::Identifier;
use crate::syntax::token::{Token, TokenInner};

impl<'s, 'd> Parser<'s, 'd> {
    pub fn parse_attribute(
        &mut self,
        hash_token: Token<'s>,
        is_global: bool,
        skip_set: &[&[TokenInner<'_>]]
    ) -> Option<Attribute<'s>> {
        debug_assert_eq!(hash_token.token_inner, TokenInner::SymHash);

        let this: &mut Parser<'s, 'd> = self;

        defer!(|this: &mut Parser<'s, 'd>| {
            this.lexer.pop_lexer_mode()
        }, this);

        this.lexer.push_lexer_mode(LexerMode::LexAttr);
        this.parse_attribute_impl(hash_token, is_global, skip_set)
    }

    fn parse_attribute_impl(
        &mut self,
        hash_token: Token<'s>,
        is_global: bool,
        skip_set: &[&[TokenInner<'_>]]
    ) -> Option<Attribute<'s>> {
        let hash_range: SourceRange = hash_token.range;

        let exclaim_range = if is_global {
            self.expect_n_consume(TokenInner::SymExclaim, skip_set)
                .expect("only appoint `is_global` when here's surely a global attribute")
                .range
        } else {
            SourceRange::unknown()
        };

        let left_bracket_range: SourceRange =
            self.expect_n_consume(TokenInner::SymLBracket, skip_set)?.range;
        let (items, right_bracket_range): (SmallVec<[AttrItem<'s>; 4]>, SourceRange) =
            self.parse_attribute_list(TokenInner::SymRBracket, skip_set)?;

        Some(Attribute {
            items,

            hash_loc: hash_range.left(),
            exclaim_loc: exclaim_range.left(),
            left_bracket_loc: left_bracket_range.left(),
            right_bracket_loc: right_bracket_range.left()
        })
    }

    pub fn parse_attribute_list(
        &mut self,
        termination: TokenInner<'_>,
        skip_set: &[&[TokenInner<'_>]]
    ) -> Option<(SmallVec<[AttrItem<'s>; 4]>, SourceRange)> {
        self.parse_list_alike_nonnull(
            Self::parse_attribute_item,
            skip_set,
            TokenInner::SymComma,
            termination,
            skip_set
        )
    }

    pub fn parse_attribute_item(&mut self, skip_set: &[&[TokenInner<'_>]]) -> Option<AttrItem<'s>> {
        let ident: Identifier<'s> = self.parse_ident().or_else(|| {
            self.skip_to_any_of(skip_set);
            None
        })?;

        match self.current_token().token_inner {
            TokenInner::SymEq => {
                let eq_token: Token<'s> = self.consume_token();
                self.parse_attribute_assign_alike_item(ident, eq_token, skip_set)
            }
            TokenInner::SymLParen => {
                let lparen_token: Token<'s> = self.consume_token();
                self.parse_attribute_call_alike_item(ident, lparen_token, skip_set)
            }
            _ => Some(AttrItem::IdentifierItem(ident))
        }
    }

    pub fn parse_attribute_assign_alike_item(
        &mut self,
        ident: Identifier<'s>,
        eq_token: Token<'s>,
        skip_set: &[&[TokenInner<'_>]]
    ) -> Option<AttrItem<'s>> {
        debug_assert_eq!(eq_token.token_inner, TokenInner::SymEq);

        let attr_value: AttrValue<'s> = self.parse_attr_value(skip_set)?;
        Some(AttrItem::AssignLikeItem(AttrAssignLikeItem {
            ident,
            value: attr_value,
            assign_loc: eq_token.range.left()
        }))
    }

    pub fn parse_attribute_call_alike_item(
        &mut self,
        ident: Identifier<'s>,
        lparen_token: Token<'s>,
        skip_set: &[&[TokenInner<'_>]]
    ) -> Option<AttrItem<'s>> {
        debug_assert_eq!(lparen_token.token_inner, TokenInner::SymLParen);
        let (items, right_paren_range): (SmallVec<[AttrItem; 4]>, SourceRange) =
            self.parse_attribute_list(TokenInner::SymRParen, skip_set)?;
        Some(AttrItem::CallLikeItem(AttrCallLikeItem {
            ident,
            args: items.into_iter().collect::<Vec<_>>(),
            left_paren_loc: lparen_token.range.left(),
            right_paren_loc: right_paren_range.left()
        }))
    }

    pub fn parse_attr_value(&mut self, skip_set: &[&[TokenInner<'_>]]) -> Option<AttrValue<'s>> {
        let range: SourceRange = self.current_token().range;
        match self.current_token().token_inner {
            TokenInner::Ident(_) => {
                let ident: Identifier<'s> = self.parse_ident().or_else(|| {
                    self.skip_to_any_of(skip_set);
                    None
                })?;
                Some(AttrValue::ident_value(ident))
            },
            TokenInner::LitInt(int_value) => Some(AttrValue::int_value(int_value, range)),
            TokenInner::LitFloat(float_value) => Some(AttrValue::float_value(float_value, range)),
            TokenInner::LitChar(char_value) => Some(AttrValue::char_value(char_value, range)),
            TokenInner::LitStr(str_value) => Some(AttrValue::string_value(str_value, range)),
            _ => {
                self.diag.borrow_mut()
                    .diag(range.left(), diag_data::err_expected_any_of_0_got_1)
                    .add_arg("todo")
                    .add_arg("todo")
                    .add_mark(range.into())
                    .build();
                self.skip_to_any_of(skip_set);
                None
            }
        }
    }
}

#[cfg(test)]
mod test {
    use std::cell::RefCell;

    use crate::diag::DiagContext;
    use crate::parse::parser::Parser;
    use crate::syntax::attr::{
        AttrAssignLikeItem,
        AttrCallLikeItem,
        AttrItem,
        AttrValue,
        AttrValueInner,
        Attribute
    };
    use crate::syntax::id::{assert_ident_unqual, assert_ident_qual};
    use crate::syntax::token::Token;

    #[test]
    fn test_parse_global_attribute() {
        let source: &str = "#![some::attribute, another = config, call(arg1, arg2, par3 = arg3)]";

        let diag: RefCell<DiagContext> = RefCell::new(DiagContext::new());
        let mut parser: Parser = Parser::new(
            0, source, &diag
        );

        let hash_token: Token = parser.consume_token();
        let attr: Attribute = parser.parse_attribute(hash_token, true, &[]).unwrap();
        assert_eq!(attr.items.len(), 3);

        if let AttrItem::IdentifierItem(ident) = &attr.items[0] {
            assert_ident_qual(ident, &["some", "attribute"]);
        } else {
            panic!("should be a identifier item");
        }

        if let AttrItem::AssignLikeItem(AttrAssignLikeItem { ident, value, .. }) = &attr.items[1] {
            assert_ident_unqual(ident, "another");
            if let AttrValue { inner: AttrValueInner::Identifier(ident), .. } = &value {
                assert_ident_unqual(ident, "config");
            } else {
                panic!("should be a identifier value");
            }
        } else {
            panic!("should be a assign like item");
        }

        if let AttrItem::CallLikeItem(AttrCallLikeItem { ident, args, .. }) = &attr.items[2] {
            assert_ident_unqual(ident, "call");
            assert_eq!(args.len(), 3);

            if let AttrItem::IdentifierItem(ident) = &args[0] {
                assert_ident_unqual(ident, "arg1");
            } else {
                panic!("should be a identifier item");
            }

            if let AttrItem::IdentifierItem(ident) = &args[1] {
                assert_ident_unqual(ident, "arg2");
            } else {
                panic!("should be a identifier item");
            }

            if let AttrItem::AssignLikeItem(AttrAssignLikeItem { ident, value, .. }) = &args[2] {
                assert_ident_unqual(ident, "par3");
                if let AttrValue { inner: AttrValueInner::Identifier(ident), .. } = &value {
                    assert_ident_unqual(ident, "arg3");
                } else {
                    panic!("should be a identifier value");
                }
            } else {
                panic!("should be a assign like item");
            }
        }
    }
}
