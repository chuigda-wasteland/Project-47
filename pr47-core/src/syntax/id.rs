//! # Concrete syntax tree of identifiers
//!
//! Identifier syntax:
//! ```text
//! identifier ::= qualifier ID
//!
//! qualifier ::= qual-list '::'
//!             | NIL
//!
//! qual-list ::= qual-list '::' ID
//!             | ID
//! ```

use smallvec::SmallVec;

use crate::syntax::token::Token;

pub enum Identifier<'a> {
    Unqual(Token<'a>),
    Qual(SmallVec<[Token<'a>; 2]>)
}

#[cfg(test)]
impl<'a> std::fmt::Debug for Identifier<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Identifier::Unqual(token) => write!(f, "{}", token.get_str_value()),
            Identifier::Qual(tokens) => {
                for i in 0..tokens.len() - 1 {
                    write!(f, "{} :: ", tokens[i].get_str_value())?;
                }
                write!(f, "{}", tokens[tokens.len() - 1])
            }
        }
    }
}

#[cfg(test)]
pub fn assert_ident_unqual(ident: &Identifier<'_>, expected: &str) {
    use crate::syntax::token::TokenInner;

    if let Identifier::Unqual(token /*: Token*/) = ident {
        if let TokenInner::Ident(actual) = token.token_inner {
            assert_eq!(actual, expected);
        } else {
            panic!("incorrect token content")
        }
    } else {
        panic!("should be an unqualified identifier")
    }
}

#[cfg(test)]
pub fn assert_ident_qual(ident: &Identifier<'_>, expected: &[&str]) {
    if let Identifier::Qual(tokens) = ident {
        tokens.iter()
            .map(Token::get_str_value)
            .zip(expected.iter())
            .for_each(|(got, expected): (&str, &&str)| assert_eq!(got, *expected));
    } else {
        panic!("should be a qualified identifier")
    }
}

#[cfg(test)]
mod test {
    use std::cell::RefCell;

    use crate::diag::DiagContext;
    use crate::parse::parser::Parser;
    use crate::syntax::id::{Identifier, assert_ident_unqual, assert_ident_qual};

    #[test]
    fn test_parse_unqual_ident() {
        let source: &str = "ablahblahblah";

        let diag: RefCell<DiagContext> = RefCell::new(DiagContext::new());
        let mut parser: Parser = Parser::new(0, source, &diag);

        let ident: Identifier = parser.parse_ident().unwrap();
        assert!(!diag.borrow().has_diag());

        dbg!(&ident);
        assert_ident_unqual(&ident, "ablahblahblah");
    }

    #[test]
    fn test_parse_qual_ident() {
        let source: &str = "ablah::blah::blahblah";

        let diag: RefCell<DiagContext> = RefCell::new(DiagContext::new());
        let mut parser: Parser = Parser::new(0, source, &diag);

        let ident: Identifier = parser.parse_ident().unwrap();
        assert!(!diag.borrow().has_diag());

        dbg!(&ident);
        assert_ident_qual(&ident, &["ablah", "blah", "blahblah"]);
    }
}
