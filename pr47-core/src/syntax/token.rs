use std::fmt::{Display, Formatter};

use crate::util::location::SourceLoc;

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

impl<'a> Display for Token<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let SourceLoc { line, col } = self.start_loc;
        let SourceLoc { line: end_line, col: end_col } = self.end_loc;

        match self.token_inner {
            TokenInner::Ident(id) => write!(f, "⟨id, {}, {}:{}⟩", id, line, col),

            TokenInner::KwdAny => write!(f, "⟨any, {}:{}⟩", line, col),
            TokenInner::KwdAs => write!(f, "⟨as, {}:{}⟩", line, col),
            TokenInner::KwdAsync => write!(f, "⟨async, {}:{}⟩", line, col),
            TokenInner::KwdAuto => write!(f, "⟨auto, {}:{}⟩", line, col),
            TokenInner::KwdAwait => write!(f, "⟨await, {}:{}⟩", line, col),
            TokenInner::KwdBool => write!(f, "⟨bool, {}:{}⟩", line, col),
            TokenInner::KwdCatch => write!(f, "⟨catch, {}:{}⟩", line, col),
            TokenInner::KwdChar => write!(f, "⟨char, {}:{}⟩", line, col),
            TokenInner::KwdConst => write!(f, "⟨const, {}:{}⟩", line, col),
            TokenInner::KwdDo => write!(f, "⟨do, {}:{}⟩", line, col),
            TokenInner::KwdElse => write!(f, "⟨else, {}:{}⟩", line, col),
            TokenInner::KwdFalse => write!(f, "⟨false, {}:{}⟩", line, col),
            TokenInner::KwdFloat => write!(f, "⟨float, {}:{}⟩", line, col),
            TokenInner::KwdFunc => write!(f, "⟨func, {}:{}⟩", line, col),
            TokenInner::KwdIf => write!(f, "⟨if, {}:{}⟩", line, col),
            TokenInner::KwdInt => write!(f, "⟨int, {}:{}⟩", line, col),
            TokenInner::KwdObject => write!(f, "⟨object, {}:{}⟩", line, col),
            TokenInner::KwdReturn => write!(f, "⟨return, {}:{}⟩", line, col),
            TokenInner::KwdSpawn => write!(f, "⟨spawn, {}:{}⟩", line, col),
            TokenInner::KwdThrow => write!(f, "⟨throw, {}:{}⟩", line, col),
            TokenInner::KwdTrue => write!(f, "⟨true, {}:{}⟩", line, col),
            TokenInner::KwdTry => write!(f, "⟨try, {}:{}⟩", line, col),
            TokenInner::KwdType => write!(f, "⟨type, {}:{}⟩", line, col),
            TokenInner::KwdTypeOf => write!(f, "⟨typeof, {}:{}⟩", line, col),
            TokenInner::KwdVar => write!(f, "⟨var, {}:{}⟩", line, col),
            TokenInner::KwdVector => write!(f, "⟨vector, {}:{}⟩", line, col),
            TokenInner::KwdWhile => write!(f, "⟨while, {}:{}⟩", line, col),

            TokenInner::LitChar(ch) => write!(f, "⟨char, '{}', {}:{}⟩", ch, line, col),
            TokenInner::LitFloat(num) => write!(f, "⟨num, {}f, {}:{}⟩", num, line, col),
            TokenInner::LitInt(num) => write!(f, "⟨num, {}i, {}:{}⟩", num, line, col),
            TokenInner::LitStr(str) => {
                if line == end_line {
                    write!(f, "⟨str, `{}`, {}:{}⟩", str, line, col)
                } else {
                    write!(f, "⟨str, `{}`, {}:{} ~ {}:{}⟩", str, line, col, end_line, end_col)
                }
            },

            TokenInner::RsvAsm => write!(f, "⟨asm, {}:{}⟩", line, col),
            TokenInner::RsvAttribute => write!(f, "⟨attribute, {}:{}⟩", line, col),
            TokenInner::RsvCkx => write!(f, "⟨ckx, {}:{}⟩", line, col),
            TokenInner::RsvRefl => write!(f, "⟨refl, {}:{}⟩", line, col),
            TokenInner::RsymAt => write!(f, "⟨@, {}:{}⟩", line, col),
            TokenInner::RsymDollar => write!(f, "⟨$, {}:{}⟩", line, col),

            TokenInner::SymAmp => write!(f, "⟨&, {}:{}⟩", line, col),
            TokenInner::SymAster => write!(f, "⟨*, {}:{}⟩", line, col),
            TokenInner::SymBackslash => write!(f, "⟨`\\`, {}:{}⟩", line, col),
            TokenInner::SymCaret => write!(f, "⟨^, {}:{}⟩", line, col),
            TokenInner::SymColon => write!(f, "⟨`:`, {}:{}⟩", line, col),
            TokenInner::SymComma => write!(f, "⟨`,`, {}:{}⟩", line, col),
            TokenInner::SymDAmp => write!(f, "⟨&&, {}:{}⟩", line, col),
            TokenInner::SymDCaret => write!(f, "⟨^^, {}:{}⟩", line, col),
            TokenInner::SymDColon => write!(f, "⟨::, {}:{}⟩", line, col),
            TokenInner::SymDEq => write!(f, "⟨≡, {}:{}⟩", line, col),
            TokenInner::SymDGt => write!(f, "⟨⇉, {}:{}⟩", line, col),
            TokenInner::SymDLt => write!(f, "⟨⇇, {}:{}⟩", line, col),
            TokenInner::SymDPipe => write!(f, "⟨||, {}:{}⟩", line, col),
            TokenInner::SymDot => write!(f, "⟨`.`, {}:{}⟩", line, col),
            TokenInner::SymEq => write!(f, "⟨=, {}:{}⟩", line, col),
            TokenInner::SymExclaim => write!(f, "⟨!, {}:{}⟩", line, col),
            TokenInner::SymGe => write!(f, "⟨`≥`, {}:{}⟩", line, col),
            TokenInner::SymGt => write!(f, "⟨`>`, {}:{}⟩", line, col),
            TokenInner::SymLBrace => write!(f, "⟨`{{`, {}:{}⟩", line, col),
            TokenInner::SymLBracket => write!(f, "⟨`[`, {}:{}⟩", line, col),
            TokenInner::SymLParen => write!(f, "⟨`(`, {}:{}⟩", line, col),
            TokenInner::SymLe => write!(f, "⟨`≤`, {}:{}⟩", line, col),
            TokenInner::SymLt => write!(f, "⟨`<`, {}:{}⟩", line, col),
            TokenInner::SymMinus => write!(f, "⟨-, {}:{}⟩", line, col),
            TokenInner::SymPercent => write!(f, "⟨%, {}:{}⟩", line, col),
            TokenInner::SymPipe => write!(f, "⟨|, {}:{}⟩", line, col),
            TokenInner::SymPlus => write!(f, "⟨+, {}:{}⟩", line, col),
            TokenInner::SymQues => write!(f, "⟨?, {}:{}⟩", line, col),
            TokenInner::SymRBrace => write!(f, "⟨`}}`, {}:{}⟩", line, col),
            TokenInner::SymRBracket => write!(f, "⟨`]`, {}:{}⟩", line, col),
            TokenInner::SymRParen => write!(f, "⟨`)`, {}:{}⟩", line, col),
            TokenInner::SymSemicolon => write!(f, "⟨`;`, {}:{}⟩", line, col),
            TokenInner::SymSharp => write!(f, "⟨#, {}:{}⟩", line, col),
            TokenInner::SymSlash => write!(f, "⟨`/`, {}:{}⟩", line, col),
            TokenInner::SymTilde => write!(f, "⟨~, {}:{}⟩", line, col)
        }
    }
}
