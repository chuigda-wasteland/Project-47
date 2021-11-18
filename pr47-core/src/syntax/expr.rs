//! # Concrete syntax tree of expressions
//!
//! Expression syntax:
//! ```text
//! expression ::= binary-expression assign-op expression
//!              | binary-expression
//!
//! binary-expression ::= binary-expression logic-op comparison-expression
//!                     | comparison-expression
//!
//! comparison-expression ::= comparison-expression compare-op bit-or-expression
//!                         | bit-or-expression
//!
//! bit-or-expression ::= bit-or-expression '|' bit-xor-expression
//!                     | bit-xor-expression
//!
//! bit-xor-expression ::= bit-xor-expression '^' bit-and-expression
//!                      | bit-and-expression
//!
//! bit-and-expression ::= bit-and-expression '&' bit-shift-expression
//!                      | bit-shift-expression
//!
//! bit-shift-expression ::= bit-shift-expression bit-shift-op additive-expression
//!                        | additive-expression
//!
//! additive-expression ::= additive-expression add-op multiplicative-expression
//!                       | multiplicative-expression
//!
//! multiplicative-expression ::= multiplicative-expression mul-op unary-expression
//!                             | unary-expression
//!
//! unary-expression ::= unary-op unary-expression
//!                    | postfix-expression
//!
//! postfix-expression ::= postfix-expression '[' binary-expression ']'
//!                      | postfix-expression '(' expression-list ')'
//!                      | postfix-expression '.' ID
//!                      | postfix-expression '.' 'await'
//!                      | postfix-expression 'as' type
//!                      | atomic-expression
//!
//! atomic-expression ::= identifier
//!                     | literal
//!                     | '(' binary-expression ')'
//!                     | intrinsic-op '(' expression-list ')'
//!
//! expression-list ::= expression-list ',' binary-expression
//!                   | binary-expression
//!                   | NIL
//!
//! assign-op ::= '=' | '+=' | '-=' | '*=' | '/=' | '%='
//!
//! logic-op ::= '&&' | '||' | '^^'
//!
//! compare-op ::= '==' | '!=' | '<' | '>' | '<=' | '>='
//!
//! bit-shift-op ::= '<<' | '>>'
//!
//! add-op ::= '+' | '-'
//!
//! mul-op ::= '*' | '/' | '%'
//!
//! unary-op ::= '+' | '-' | '!' | '~'
//!
//! intrinsic-op ::= TODO define intrinsics
//! ```

use crate::diag::location::{SourceLoc, SourceRange};
use crate::syntax::id::Identifier;
use crate::syntax::token::Token;
use crate::syntax::ty::ConcreteType;

#[cfg(test)] use std::fmt::{Debug, Formatter};

pub enum ConcreteExpr<'a> {
    LiteralExpr(ConcreteLiteralExpr<'a>),
    IdRefExpr(Identifier<'a>),
    UnaryExpr(ConcreteUnaryExpr<'a>),
    BinaryExpr(ConcreteBinaryExpr<'a>),
    FuncCallExpr(ConcreteFuncCallExpr<'a>),
    SubscriptExpr(ConcreteSubscriptExpr<'a>),
    FieldRefExpr(ConcreteFieldRefExpr<'a>),
    AsExpr(ConcreteAsExpr<'a>),
    AwaitExpr(ConcreteAwaitExpr<'a>),
    ParenthesizedExpr(ConcreteParenthesizedExpr<'a>),
}

#[cfg(test)]
impl<'a> Debug for ConcreteExpr<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ConcreteExpr::LiteralExpr(expr) => expr.fmt(f),
            ConcreteExpr::IdRefExpr(expr) => expr.fmt(f),
            ConcreteExpr::UnaryExpr(expr) => expr.fmt(f),
            ConcreteExpr::BinaryExpr(expr) => expr.fmt(f),
            ConcreteExpr::FuncCallExpr(expr) => expr.fmt(f),
            ConcreteExpr::SubscriptExpr(expr) => expr.fmt(f),
            ConcreteExpr::FieldRefExpr(expr) => expr.fmt(f),
            ConcreteExpr::AsExpr(expr) => expr.fmt(f),
            ConcreteExpr::AwaitExpr(expr) => expr.fmt(f),
            ConcreteExpr::ParenthesizedExpr(expr) => expr.fmt(f)
        }
    }
}

#[cfg_attr(test, derive(Debug))]
pub struct ConcreteLiteralExpr<'a> {
    pub content: LiteralExprContent<'a>,
    pub range: SourceRange
}

#[cfg_attr(test, derive(Debug))]
pub enum LiteralExprContent<'a> {
    Int(i64),
    Float(f64),
    Char(char),
    String(&'a str),
    Boolean(bool)
}

impl<'a> ConcreteLiteralExpr<'a> {
    pub fn new_lit_int(lit: i64, range: SourceRange) -> Self {
        ConcreteLiteralExpr {
            content: LiteralExprContent::Int(lit),
            range
        }
    }

    pub fn new_lit_float(lit: f64, range: SourceRange) -> Self {
        ConcreteLiteralExpr {
            content: LiteralExprContent::Float(lit), range
        }
    }

    pub fn new_lit_char(lit: char, range: SourceRange) -> Self {
        ConcreteLiteralExpr {
            content: LiteralExprContent::Char(lit), range
        }
    }

    pub fn new_lit_str(lit: &'a str, range: SourceRange) -> Self {
        ConcreteLiteralExpr {
            content: LiteralExprContent::String(lit), range
        }
    }

    pub fn new_lit_bool(lit: bool, range: SourceRange) -> Self {
        ConcreteLiteralExpr {
            content: LiteralExprContent::Boolean(lit), range
        }
    }
}

#[cfg_attr(test, derive(Debug))]
pub struct ConcreteStringLiteralExpr<'a> {
    pub value: &'a str,
    pub range: SourceRange
}

#[cfg_attr(test, derive(Debug))]
pub struct ConcreteUnaryExpr<'a> {
    pub op: Token<'a>,
    pub operand: Box<ConcreteExpr<'a>>,
}

#[cfg_attr(test, derive(Debug))]
pub struct ConcreteBinaryExpr<'a> {
    pub op: Token<'a>,
    pub lhs: Box<ConcreteExpr<'a>>,
    pub rhs: Box<ConcreteExpr<'a>>,
}

#[cfg_attr(test, derive(Debug))]
pub struct ConcreteFuncCallExpr<'a> {
    pub func: Box<ConcreteExpr<'a>>,
    pub args: Vec<ConcreteExpr<'a>>,

    pub lparen_loc: SourceLoc,
    pub rparen_loc: SourceLoc
}

#[cfg_attr(test, derive(Debug))]
pub struct ConcreteSubscriptExpr<'a> {
    pub base: Box<ConcreteExpr<'a>>,
    pub idx: Box<ConcreteExpr<'a>>,

    pub lbracket_loc: SourceLoc,
    pub rbracket_loc: SourceLoc
}

#[cfg_attr(test, derive(Debug))]
pub struct ConcreteFieldRefExpr<'a> {
    pub base: Box<ConcreteExpr<'a>>,
    pub id: Identifier<'a>,

    pub dot_loc: SourceLoc
}

pub struct ConcreteAwaitExpr<'a> {
    pub base: Box<ConcreteExpr<'a>>,
    pub dot_loc: SourceLoc,
    pub await_range: SourceRange
}

pub struct ConcreteAsExpr<'a> {
    pub operand: Box<ConcreteExpr<'a>>,
    pub dest_type: ConcreteType<'a>,

    pub as_range: SourceRange
}

pub struct ConcreteParenthesizedExpr<'a> {
    pub inner: Box<ConcreteExpr<'a>>,
    pub lparen_loc: SourceLoc,
    pub rparen_loc: SourceLoc
}

#[cfg(test)]
impl<'a> Debug for ConcreteAwaitExpr<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConcreteAwaitExpr")
            .field("base", self.base.as_ref())
            .finish()
    }
}

#[cfg(test)]
impl<'a> Debug for ConcreteAsExpr<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConcreteAsExpr")
            .field("operand", self.operand.as_ref())
            .field("dest_type", &self.dest_type)
            .finish()
    }
}

#[cfg(test)]
impl<'a> Debug for ConcreteParenthesizedExpr<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConcreteParenthesizedExpr")
            .field("inner", &self.inner)
            .finish()
    }
}
