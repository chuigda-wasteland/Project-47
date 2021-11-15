//! # Concrete syntax tree of types
//!
//! Type syntax:
//! ```text
//! type ::= primitive-type
//!        | generic-type
//!        | deduced-type
//!        | user-type
//!
//! primitive-type ::= 'any' | 'char' | 'float' | 'int' | 'object' | 'string' | 'void'
//!
//! generic-type ::= 'vector' '<' generic-type-parameter '>'
//!
//! generic-type-parameter-list ::= generic-type-parameter-list ',' type
//!                               | type
//!
//! deduced-type ::= 'auto'
//!
//! user-type ::= identifier
//! ```

use smallvec::SmallVec;

use crate::diag::location::{SourceLoc, SourceRange};
use crate::syntax::id::Identifier;
use crate::syntax::token::Token;

#[cfg_attr(debug_assertions, derive(Debug))]
pub enum ConcreteType<'a> {
    PrimitiveType(Token<'a>),
    GenericType(Box<ConcreteGenericType<'a>>),
    DeducedType(SourceRange),
    UserType(Identifier<'a>)
}

#[cfg_attr(debug_assertions, derive(Debug))]
pub struct ConcreteGenericType<'a> {
    pub base: Token<'a>,
    pub inner: SmallVec<[ConcreteType<'a>; 2]>,
    pub left_angle: SourceLoc,
    pub right_angle: SourceLoc
}
