use super::Parser;

use smallvec::{SmallVec, smallvec};
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
        is_global: bool,
        skip_set: &[&[TokenInner<'_>]]
    ) -> Option<Attribute<'s>> {
        let this: &mut Parser<'s, 'd> = self;

        defer!(|this: &mut Parser<'s, 'd>| {
            this.lexer.pop_lexer_mode()
        }, this);

        this.lexer.push_lexer_mode(LexerMode::LexAttr);
        this.parse_attribute_impl(is_global, skip_set)
    }

    fn parse_attribute_impl(
        &mut self,
        is_global: bool,
        skip_set: &[&[TokenInner<'_>]]
    ) -> Option<Attribute<'s>> {
        let hash_range: SourceRange = self.current_token().range;
        self.consume_token();

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
        let mut items: SmallVec<[AttrItem<'s>; 4]> = smallvec![];
        let termination_range: SourceRange = loop {
            if self.current_token().is_eoi() {
                self.diag_unexpected_eoi(self.current_token().range);
                return None;
            }

            let attr_item: AttrItem<'s> = self.parse_attribute_item(skip_set)?;
            items.push(attr_item);

            if self.m_current_token.token_inner == TokenInner::SymComma {
                self.consume_token();
            } else {
                break self.expect_n_consume(termination, skip_set)?.range;
            }
        };
        Some((items, termination_range))
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
    #[test]
    fn test_parse_global_attribute() {

    }
}
