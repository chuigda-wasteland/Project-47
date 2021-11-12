//! # attribute parsing
//!
//! Attribute syntax:
//! ```text
//! global-attribute ::= '#' '!' '[' attribute-list ']'
//!
//! attribute ::= '#' '[' attribute-list ']'
//!
//! attribute-list ::= attribute-list ',' attribute-item
//!                  | attribute-item
//!
//! attribute-item ::= attribute-identifier-item
//!                  | attribute-assign-like-item
//!                  | attribute-call-alike-item
//!
//! attribute-identifier-item ::= identifier
//!
//! attribute-assign-like-item ::= identifier '=' attribute-value
//!
//! attribute-call-alike-item ::= identifier '(' attribute-list ')'
//!
//! attribute-value ::= identifier
//!                   | literal
//! ```

use smallvec::SmallVec;

use crate::diag::location::SourceLoc;
use crate::syntax::id::Identifier;

pub struct Attribute<'a> {
    pub items: SmallVec<[AttrItem<'a>; 4]>,

    pub hash_loc: SourceLoc,
    pub exclaim_loc: SourceLoc,
    pub left_bracket_loc: SourceLoc,
    pub right_bracket_loc: SourceLoc
}

pub enum AttrItem<'a> {
    IdentifierItem(Identifier<'a>),
    AssignLikeItem(AttrAssignLikeItem<'a>),
    CallLikeItem(AttrCallLikeItem<'a>)
}

pub struct AttrAssignLikeItem<'a> {
    pub ident: Identifier<'a>,
    pub value: AttrValue<'a>,

    pub assign_loc: SourceLoc,
}

pub struct AttrCallLikeItem<'a> {
    pub ident: Identifier<'a>,
    pub args: Vec<[AttrItem<'a>; 4]>,

    pub left_paren_loc: SourceLoc,
    pub right_paren_loc: SourceLoc,
}

pub enum AttrValue<'a> {
    Identifier(Identifier<'a>),
    IntLiteral(i64),
    FloatLiteral(f64),
    CharLiteral(char),
    StringLiteral(&'a str)
}
