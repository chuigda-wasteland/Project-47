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

#[cfg_attr(debug_assertions, derive(Debug))]
pub enum Identifier<'a> {
    Unqual(Token<'a>),
    Qual(SmallVec<[Token<'a>; 2]>)
}
