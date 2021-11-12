use super::{Parser, TOP_LEVEL_FIRST};

use smallvec::{SmallVec, smallvec};

use crate::diag::location::SourceRange;
use crate::syntax::attr::{Attribute, AttrItem};
use crate::syntax::token::TokenInner;

impl<'a, 'b> Parser<'a, 'b> {
    pub fn parse_attribute(&mut self, is_global: bool) -> Option<Attribute<'a>> {
        let hash_range: SourceRange = self.current_token().range;
        self.consume_token();

        let exclaim_range = if is_global {
            self.expect_n_consume(TokenInner::SymExclaim, TOP_LEVEL_FIRST)
                .expect("only appoint `is_global` when here's surely a global attribute")
                .range
        } else {
            SourceRange::unknown()
        };

        let left_bracket_range: SourceRange =
            self.expect_n_consume(TokenInner::SymLBracket, TOP_LEVEL_FIRST)?.range;

        let mut items: SmallVec<[AttrItem; 4]> = smallvec![];
        let right_bracket_range: SourceRange = loop {
            if self.current_token().is_eoi() {
                self.diag_unexpected_eoi(self.current_token().range);
                return None;
            }

            if let Some(attr) = self.parse_attribute_item() {
                items.push(attr);
                if self.m_current_token.token_inner == TokenInner::SymComma {
                    self.consume_token();
                } else {
                    break self.expect_n_consume(TokenInner::SymRBracket, &[])?.range;
                }
            } else {
                return None;
            }
        };

        Some(Attribute {
            items,

            hash_loc: hash_range.left(),
            exclaim_loc: exclaim_range.left(),
            left_bracket_loc: left_bracket_range.left(),
            right_bracket_loc: right_bracket_range.left()
        })
    }

    pub fn parse_attribute_item(&mut self) -> Option<AttrItem<'a>> {
        todo!()
    }
}
