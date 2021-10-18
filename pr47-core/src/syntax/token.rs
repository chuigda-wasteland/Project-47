use std::fmt::{Display, Formatter};

use crate::util::location::SourceLoc;
use smallvec::alloc::fmt::Debug;

#[derive(Clone, Copy)]
pub enum TokenInner<'a> {
    Ident(&'a str),

    KwdAny,
    KwdAs,
    KwdAsync,
    KwdAuto,
    KwdAwait,
    KwdBool,
    KwdCatch,
    KwdChar,
    KwdConst,
    KwdDo,
    KwdElse,
    KwdFalse,
    KwdFloat,
    KwdFunc,
    KwdIf,
    KwdImport,
    KwdInt,
    KwdObject,
    KwdReturn,
    KwdSpawn,
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
    SymDGt,
    SymDLt,
    SymDPipe,
    SymDPlus,
    SymDMinus,
    SymDot,
    SymEq,
    SymExclaim,
    SymGe,
    SymGt,
    SymLBrace,
    SymLBracket,
    SymLParen,
    SymLe,
    SymLt,
    SymMinus,
    SymNe,
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

impl<'a> TokenInner<'a> {
    pub fn is_reserved(&self) -> bool {
        use TokenInner::*;
        match *self {
            RsvAsm | RsvAttribute | RsvCkx | RsvRefl | RsymAt | RsymDollar => true,
            _ => false
        }
    }
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

impl<'a> Display for Token<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let SourceLoc { line, col } = self.start_loc;
        let SourceLoc { line: end_line, col: end_col } = self.end_loc;

        use TokenInner::*;
        match self.token_inner {
            Ident(id) => write!(f, "⟨id, {}, {}:{}⟩", id, line, col),

            KwdAny => write!(f, "⟨any, {}:{}⟩", line, col),
            KwdAs => write!(f, "⟨as, {}:{}⟩", line, col),
            KwdAsync => write!(f, "⟨async, {}:{}⟩", line, col),
            KwdAuto => write!(f, "⟨auto, {}:{}⟩", line, col),
            KwdAwait => write!(f, "⟨await, {}:{}⟩", line, col),
            KwdBool => write!(f, "⟨bool, {}:{}⟩", line, col),
            KwdCatch => write!(f, "⟨catch, {}:{}⟩", line, col),
            KwdChar => write!(f, "⟨char, {}:{}⟩", line, col),
            KwdConst => write!(f, "⟨const, {}:{}⟩", line, col),
            KwdDo => write!(f, "⟨do, {}:{}⟩", line, col),
            KwdElse => write!(f, "⟨else, {}:{}⟩", line, col),
            KwdFalse => write!(f, "⟨false, {}:{}⟩", line, col),
            KwdFloat => write!(f, "⟨float, {}:{}⟩", line, col),
            KwdFunc => write!(f, "⟨func, {}:{}⟩", line, col),
            KwdIf => write!(f, "⟨if, {}:{}⟩", line, col),
            KwdImport => write!(f, "⟨import, {}:{}⟩", line, col),
            KwdInt => write!(f, "⟨int, {}:{}⟩", line, col),
            KwdObject => write!(f, "⟨object, {}:{}⟩", line, col),
            KwdReturn => write!(f, "⟨return, {}:{}⟩", line, col),
            KwdSpawn => write!(f, "⟨spawn, {}:{}⟩", line, col),
            KwdThrow => write!(f, "⟨throw, {}:{}⟩", line, col),
            KwdTrue => write!(f, "⟨true, {}:{}⟩", line, col),
            KwdTry => write!(f, "⟨try, {}:{}⟩", line, col),
            KwdType => write!(f, "⟨type, {}:{}⟩", line, col),
            KwdTypeOf => write!(f, "⟨typeof, {}:{}⟩", line, col),
            KwdVar => write!(f, "⟨var, {}:{}⟩", line, col),
            KwdVector => write!(f, "⟨vector, {}:{}⟩", line, col),
            KwdWhile => write!(f, "⟨while, {}:{}⟩", line, col),

            LitChar(ch) => write!(f, "⟨char, '{}', {}:{}⟩", ch, line, col),
            LitFloat(num) => write!(f, "⟨num, {}f, {}:{}⟩", num, line, col),
            LitInt(num) => write!(f, "⟨num, {}i, {}:{}⟩", num, line, col),
            LitStr(str) => {
                if line == end_line {
                    write!(f, "⟨str, `{}`, {}:{}⟩", str, line, col)
                } else {
                    write!(f, "⟨str, `{}`, {}:{} ~ {}:{}⟩", str, line, col, end_line, end_col)
                }
            },

            RsvAsm => write!(f, "⟨asm, {}:{}⟩", line, col),
            RsvAttribute => write!(f, "⟨attribute, {}:{}⟩", line, col),
            RsvCkx => write!(f, "⟨ckx, {}:{}⟩", line, col),
            RsvRefl => write!(f, "⟨refl, {}:{}⟩", line, col),
            RsymAt => write!(f, "⟨@, {}:{}⟩", line, col),
            RsymDollar => write!(f, "⟨$, {}:{}⟩", line, col),

            SymAmp => write!(f, "⟨&, {}:{}⟩", line, col),
            SymAster => write!(f, "⟨*, {}:{}⟩", line, col),
            SymBackslash => write!(f, "⟨`\\`, {}:{}⟩", line, col),
            SymCaret => write!(f, "⟨^, {}:{}⟩", line, col),
            SymColon => write!(f, "⟨`:`, {}:{}⟩", line, col),
            SymComma => write!(f, "⟨`,`, {}:{}⟩", line, col),
            SymDAmp => write!(f, "⟨&&, {}:{}⟩", line, col),
            SymDCaret => write!(f, "⟨^^, {}:{}⟩", line, col),
            SymDColon => write!(f, "⟨::, {}:{}⟩", line, col),
            SymDEq => write!(f, "⟨≡, {}:{}⟩", line, col),
            SymDGt => write!(f, "⟨⇉, {}:{}⟩", line, col),
            SymDLt => write!(f, "⟨⇇, {}:{}⟩", line, col),
            SymDPipe => write!(f, "⟨||, {}:{}⟩", line, col),
            SymDPlus => write!(f, "⟨++, {}:{}⟩", line, col),
            SymDMinus => write!(f, "⟨--, {}:{}⟩", line, col),
            SymDot => write!(f, "⟨`.`, {}:{}⟩", line, col),
            SymEq => write!(f, "⟨=, {}:{}⟩", line, col),
            SymExclaim => write!(f, "⟨!, {}:{}⟩", line, col),
            SymGe => write!(f, "⟨`≥`, {}:{}⟩", line, col),
            SymGt => write!(f, "⟨`>`, {}:{}⟩", line, col),
            SymLBrace => write!(f, "⟨`{{`, {}:{}⟩", line, col),
            SymLBracket => write!(f, "⟨`[`, {}:{}⟩", line, col),
            SymLParen => write!(f, "⟨`(`, {}:{}⟩", line, col),
            SymLe => write!(f, "⟨`≤`, {}:{}⟩", line, col),
            SymLt => write!(f, "⟨`<`, {}:{}⟩", line, col),
            SymMinus => write!(f, "⟨-, {}:{}⟩", line, col),
            SymNe => write!(f, "⟨!=, {}:{}⟩", line, col),
            SymPercent => write!(f, "⟨%, {}:{}⟩", line, col),
            SymPipe => write!(f, "⟨|, {}:{}⟩", line, col),
            SymPlus => write!(f, "⟨+, {}:{}⟩", line, col),
            SymQues => write!(f, "⟨?, {}:{}⟩", line, col),
            SymRBrace => write!(f, "⟨`}}`, {}:{}⟩", line, col),
            SymRBracket => write!(f, "⟨`]`, {}:{}⟩", line, col),
            SymRParen => write!(f, "⟨`)`, {}:{}⟩", line, col),
            SymSemicolon => write!(f, "⟨`;`, {}:{}⟩", line, col),
            SymSharp => write!(f, "⟨#, {}:{}⟩", line, col),
            SymSlash => write!(f, "⟨`/`, {}:{}⟩", line, col),
            SymTilde => write!(f, "⟨~, {}:{}⟩", line, col)
        }
    }
}

impl<'a> Debug for Token<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}
