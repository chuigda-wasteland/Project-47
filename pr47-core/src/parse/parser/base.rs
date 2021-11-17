use super::Parser;

use smallvec::{SmallVec, smallvec};

use crate::diag::location::SourceRange;
use crate::syntax::id::Identifier;
use crate::syntax::token::{Token, TokenInner};
use crate::util::append::Appendable;

impl<'s, 'd> Parser<'s, 'd> {
    pub fn parse_ident(&mut self) -> Option<Identifier<'s>> {
        let token: Token<'s> = self.expect_n_consume(TokenInner::Ident(""), &[])?;
        if self.current_token().token_inner != TokenInner::SymDColon {
            return Some(Identifier::Unqual(token))
        }

        let mut token_buffer: SmallVec<[Token<'s>; 2]> = smallvec![token];
        while self.current_token().token_inner == TokenInner::SymDColon {
            self.consume_token();
            let token: Token<'s> = self.expect_n_consume(TokenInner::Ident(""), &[])?;
            token_buffer.push(token);
        }

        Some(Identifier::Qual(token_buffer))
    }

    pub fn parse_unqual_ident(&mut self) -> Option<Identifier<'s>> {
        let token: Token<'s> = self.expect_n_consume(TokenInner::Ident(""), &[])?;
        if self.current_token().token_inner == TokenInner::SymDColon {
            todo!("report error and recovery")
        }

        Some(Identifier::Unqual(token))
    }

    pub fn parse_list_alike<I, F, V>(
        &mut self,
        parse_item_fn: F,
        item_skip_set: &[&[TokenInner<'_>]],
        separation: TokenInner<'_>,
        termination: TokenInner<'_>,
        skip_set: &[&[TokenInner<'_>]]
    ) -> Option<(V, SourceRange)>
        where I: 's,
              F: for<'r> Fn(&mut Self, &[&[TokenInner<'r>]]) -> Option<I>,
              V: Appendable<Item = I> + Default
    {
        let mut items: V = V::default();
        loop {
            if self.current_token().is_eoi() {
                self.diag_unexpected_eoi(self.current_token().range);
                return None
            }

            if self.current_token().token_inner == termination {
                break;
            }

            let item: I = parse_item_fn(self, item_skip_set).or_else(|| {
                self.skip_to_any_of(skip_set);
                None
            })?;

            items.push_back(item);

            if self.current_token().token_inner == separation {
                self.consume_token();
            }
        }

        Some((items, self.consume_token().range))
    }

    pub fn parse_list_alike_nonnull<I, F, V>(
        &mut self,
        parse_item_fn: F,
        item_skip_set: &[&[TokenInner<'_>]],
        separation: TokenInner<'_>,
        termination: TokenInner<'_>,
        skip_set: &[&[TokenInner<'_>]]
    ) -> Option<(V, SourceRange)>
        where I: 's,
              F: for<'r> Fn(&mut Self, &[&[TokenInner<'r>]]) -> Option<I>,
              V: Appendable<Item = I> + Default
    {
        let first_item: I = parse_item_fn(self, item_skip_set).or_else(|| {
            self.skip_to_any_of(skip_set);
            None
        })?;

        if self.current_token().token_inner == separation {
            self.consume_token();
            let (rest, termination_range): (SmallVec<[I; 4]>, SourceRange) = self.parse_list_alike(
                parse_item_fn,
                item_skip_set,
                separation,
                termination,
                skip_set
            )?;

            let mut ret = V::default();
            ret.push_back(first_item);
            for item in rest {
                ret.push_back(item);
            }

            Some((ret, termination_range))
        } else {
            let termination_range: SourceRange =
                self.expect_n_consume(termination, skip_set)?.range;
            let mut ret = V::default();
            ret.push_back(first_item);
            Some((ret, termination_range))
        }
    }
}

#[cfg(test)]
mod test {
    use std::cell::RefCell;

    use crate::diag::DiagContext;
    use crate::diag::location::SourceRange;
    use crate::parse::parser::Parser;
    use crate::syntax::id::{Identifier, assert_ident_unqual};
    use crate::syntax::token::TokenInner;

    impl<'s, 'd> Parser<'s, 'd> {
        pub fn test_parse_unqual_ident(&mut self, _skip_set: &[&[TokenInner<'_>]])
            -> Option<Identifier<'s>>
        {
            self.parse_unqual_ident()
        }
    }

    #[test]
    fn test_parse_list_alike() {
        let diag: RefCell<DiagContext> = RefCell::new(DiagContext::new());
        let mut parser: Parser = Parser::new(0, "(a, b, c, d)", &diag);

        parser.expect_n_consume(TokenInner::SymLParen, &[]).unwrap();
        let (id_list, _rparen_range): (Vec<Identifier>, SourceRange) = parser.parse_list_alike(
            Parser::test_parse_unqual_ident,
            &[],
            TokenInner::SymComma,
            TokenInner::SymRParen,
            &[]
        ).unwrap();

        assert_eq!(id_list.len(), 4);
        assert_ident_unqual(&id_list[0], "a");
        assert_ident_unqual(&id_list[1], "b");
        assert_ident_unqual(&id_list[2], "c");
        assert_ident_unqual(&id_list[3], "d");
    }

    #[test]
    fn test_parse_list_alike_empty() {
        let diag: RefCell<DiagContext> = RefCell::new(DiagContext::new());
        let mut parser: Parser = Parser::new(0, "[]", &diag);

        parser.expect_n_consume(TokenInner::SymLBracket, &[]).unwrap();
        let (id_list, _rparen_range): (Vec<Identifier>, SourceRange) = parser.parse_list_alike(
            Parser::test_parse_unqual_ident,
            &[],
            TokenInner::SymComma,
            TokenInner::SymRBracket,
            &[]
        ).unwrap();

        assert_eq!(id_list.len(), 0);
    }

    #[test]
    fn test_parse_list_alike_trailing_comma() {
        let diag: RefCell<DiagContext> = RefCell::new(DiagContext::new());
        let mut parser: Parser = Parser::new(0, "(a, b, c, d,)", &diag);

        parser.expect_n_consume(TokenInner::SymLParen, &[]).unwrap();
        let (id_list, _rparen_range): (Vec<Identifier>, SourceRange) = parser.parse_list_alike(
            Parser::test_parse_unqual_ident,
            &[],
            TokenInner::SymComma,
            TokenInner::SymRParen,
            &[]
        ).unwrap();

        assert_eq!(id_list.len(), 4);
        assert_ident_unqual(&id_list[0], "a");
        assert_ident_unqual(&id_list[1], "b");
        assert_ident_unqual(&id_list[2], "c");
        assert_ident_unqual(&id_list[3], "d");
    }

    #[test]
    #[should_panic]
    fn test_parse_list_early_eoi() {
        let diag: RefCell<DiagContext> = RefCell::new(DiagContext::new());
        let mut parser: Parser = Parser::new(0, "(a, b, c", &diag);

        parser.expect_n_consume(TokenInner::SymLParen, &[]).unwrap();
        let _: Option<(Vec<_>, _)> = parser.parse_list_alike(
            Parser::test_parse_unqual_ident,
            &[],
            TokenInner::SymComma,
            TokenInner::SymRParen,
            &[]
        );
    }

    #[test]
    #[should_panic]
    fn test_parse_list_early_eoi2() {
        let diag: RefCell<DiagContext> = RefCell::new(DiagContext::new());
        let mut parser: Parser = Parser::new(0, "(a, b, c,", &diag);

        parser.expect_n_consume(TokenInner::SymLParen, &[]).unwrap();
        let _: Option<(Vec<_>, _)> = parser.parse_list_alike(
            Parser::test_parse_unqual_ident,
            &[],
            TokenInner::SymComma,
            TokenInner::SymRParen,
            &[]
        );
    }

    #[test]
    #[should_panic]
    fn test_parse_list_alike_duplicate_comma() {
        let diag: RefCell<DiagContext> = RefCell::new(DiagContext::new());
        let mut parser: Parser = Parser::new(0, "(a, b, c,, d)", &diag);

        parser.expect_n_consume(TokenInner::SymLParen, &[]).unwrap();
        let _: Option<(Vec<_>, _)> = parser.parse_list_alike(
            Parser::test_parse_unqual_ident,
            &[],
            TokenInner::SymComma,
            TokenInner::SymRParen,
            &[]
        );
    }

    #[test]
    #[should_panic]
    fn test_parse_list_alike_nonnull_empty() {
        let diag: RefCell<DiagContext> = RefCell::new(DiagContext::new());
        let mut parser: Parser = Parser::new(0, "[]", &diag);

        parser.expect_n_consume(TokenInner::SymLBracket, &[]).unwrap();
        let _: Option<(Vec<_>, _)> = parser.parse_list_alike_nonnull(
            Parser::test_parse_unqual_ident,
            &[],
            TokenInner::SymComma,
            TokenInner::SymRBracket,
            &[]
        );
    }
}
