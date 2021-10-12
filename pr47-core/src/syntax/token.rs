use std::fmt::{Debug, Formatter};

use crate::util::location::SourceLoc;

pub enum TokenInner<'a> {
    Ident(&'a str),

    KwdAny,
    KwdAs,
    KwdAuto,
    KwdBool,
    KwdCatch,
    KwdChar,
    KwdConst,
    KwdDo,
    KwdFalse,
    KwdFloat,
    KwdFunc,
    KwdInt,
    KwdObject,
    KwdReturn,
    KwdThrow,
    KwdTrue,
    KwdTry,
    KwdType,
    KwdTypeOf,
    KwdVar,
    KwdVector,
    KwdWhile,

    LitChar(char),
    LitFloat(f64),
    LitInt(i64),
    LitStr(&'a str),

    RsvAsm,
    RsvAttribute,
    RsvCkx,
    RsvRefl,

    RsymAt,
    RsymDollar,

    SymAmp,
    SymAster,
    SymBackslash,
    SymCaret,
    SymColon,
    SymComma,
    SymDAmp,
    SymDCaret,
    SymDColon,
    SymDEq,
    SymDPipe,
    SymDot,
    SymEq,
    SymExclaim,
    SymGt,
    SymLBrace,
    SymLBracket,
    SymLParen,
    SymLt,
    SymMinus,
    SymPercent,
    SymPipe,
    SymPlus,
    SymQues,
    SymRBrace,
    SymRBracket,
    SymRParen,
    SymSemicolon,
    SymSharp,
    SymSlash,
    SymTilde
}

pub struct Token<'a> {
    pub token_inner: TokenInner<'a>,
    pub start_loc: SourceLoc,
    pub end_loc: SourceLoc
}

impl<'a> Token<'a> {
    pub fn new(token_inner: TokenInner<'a>, start_loc: SourceLoc, end_loc: SourceLoc) -> Self {
        Self { token_inner, start_loc, end_loc }
    }

    pub fn new_lit_int(lit: i64, start_loc: SourceLoc, end_loc: SourceLoc) -> Self {
        Self::new(TokenInner::LitInt(lit), start_loc, end_loc)
    }

    pub fn new_lit_float(lit: f64, start_loc: SourceLoc, end_loc: SourceLoc) -> Self {
        Self::new(TokenInner::LitFloat(lit), start_loc, end_loc)
    }

    pub fn new_lit_char(lit: char, start_loc: SourceLoc, end_loc: SourceLoc) -> Self {
        Self::new(TokenInner::LitChar(lit), start_loc, end_loc)
    }

    pub fn new_lit_str(lit: &'a str, start_loc: SourceLoc, end_loc: SourceLoc) -> Self {
        Self::new(TokenInner::LitStr(lit), start_loc, end_loc)
    }

    pub fn new_id(id: &'a str, start_loc: SourceLoc, end_loc: SourceLoc) -> Self {
        Self::new(TokenInner::Ident(id), start_loc, end_loc)
    }
}

impl<'a> Debug for Token<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let SourceLoc { line, col } = self.start_loc;
        let SourceLoc { line: end_line, col: end_col } = self.end_loc;

        match self.token_inner {
            TokenInner::Ident(id) => write!(
                f, "Token::Ident({}@({},{})~({},{}))", id, line, col, end_line, end_col
            ),
            TokenInner::KwdAny => write!(
                f, "Token::Any(@({},{})~({},{}))", line, col, end_line, end_col
            ),
            _ => todo!()
        }
    }
}
