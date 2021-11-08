use std::fmt::{Display, Formatter};

use smallvec::alloc::fmt::Debug;

use crate::diag::location::SourceRange;

#[derive(Clone, Copy)]
pub enum TokenInner<'a> {
    Ident(&'a str),

    KwdAny,
    KwdAs,
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
    KwdOpen,
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
    RsvRequire,

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
    pub range: SourceRange
}

impl<'a> Token<'a> {
    pub fn new(token_inner: TokenInner<'a>, range: SourceRange) -> Self {
        Self { token_inner, range }
    }

    pub fn new_lit_int(lit: i64, range: SourceRange) -> Self {
        Self::new(TokenInner::LitInt(lit), range)
    }

    pub fn new_lit_float(lit: f64, range: SourceRange) -> Self {
        Self::new(TokenInner::LitFloat(lit), range)
    }

    pub fn new_lit_char(lit: char, range: SourceRange) -> Self {
        Self::new(TokenInner::LitChar(lit), range)
    }

    pub fn new_lit_str(lit: &'a str, range: SourceRange) -> Self {
        Self::new(TokenInner::LitStr(lit), range)
    }

    pub fn new_id(id: &'a str, range: SourceRange) -> Self {
        Self::new(TokenInner::Ident(id), range)
    }
}

impl<'a> Display for Token<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use TokenInner::*;
        match self.token_inner {
            Ident(id) => write!(f, "⟨id, {}⟩", id),

            KwdAny => write!(f, "⟨any⟩"),
            KwdAs => write!(f, "⟨as⟩"),
            KwdAuto => write!(f, "⟨auto⟩"),
            KwdAwait => write!(f, "⟨await⟩"),
            KwdBool => write!(f, "⟨bool⟩"),
            KwdCatch => write!(f, "⟨catch⟩"),
            KwdChar => write!(f, "⟨char⟩"),
            KwdConst => write!(f, "⟨const⟩"),
            KwdDo => write!(f, "⟨do⟩"),
            KwdElse => write!(f, "⟨else⟩"),
            KwdFalse => write!(f, "⟨false⟩"),
            KwdFloat => write!(f, "⟨float⟩"),
            KwdFunc => write!(f, "⟨func⟩"),
            KwdIf => write!(f, "⟨if⟩"),
            KwdImport => write!(f, "⟨import⟩"),
            KwdInt => write!(f, "⟨int⟩"),
            KwdObject => write!(f, "⟨object⟩"),
            KwdOpen => write!(f, "⟨open⟩"),
            KwdReturn => write!(f, "⟨return⟩"),
            KwdSpawn => write!(f, "⟨spawn⟩"),
            KwdThrow => write!(f, "⟨throw⟩"),
            KwdTrue => write!(f, "⟨true⟩"),
            KwdTry => write!(f, "⟨try⟩"),
            KwdType => write!(f, "⟨type⟩"),
            KwdTypeOf => write!(f, "⟨typeof⟩"),
            KwdVar => write!(f, "⟨var⟩"),
            KwdVector => write!(f, "⟨vector⟩"),
            KwdWhile => write!(f, "⟨while⟩"),

            LitChar(ch) => write!(f, "⟨char, '{}'⟩", ch),
            LitFloat(num) => write!(f, "⟨num, {}f⟩", num),
            LitInt(num) => write!(f, "⟨num, {}i⟩", num),
            LitStr(str) => write!(f, "⟨str, `{}`⟩", str),

            RsvAsm => write!(f, "⟨asm⟩"),
            RsvAttribute => write!(f, "⟨attribute⟩"),
            RsvCkx => write!(f, "⟨ckx⟩"),
            RsvRefl => write!(f, "⟨refl⟩"),
            RsvRequire => write!(f, "⟨require⟩"),
            RsymAt => write!(f, "⟨@⟩"),
            RsymDollar => write!(f, "⟨$⟩"),

            SymAmp => write!(f, "⟨&⟩"),
            SymAster => write!(f, "⟨*⟩"),
            SymBackslash => write!(f, "⟨`\\`⟩"),
            SymCaret => write!(f, "⟨^⟩"),
            SymColon => write!(f, "⟨`:`⟩"),
            SymComma => write!(f, "⟨`,`⟩"),
            SymDAmp => write!(f, "⟨&&⟩"),
            SymDCaret => write!(f, "⟨^^⟩"),
            SymDColon => write!(f, "⟨::⟩"),
            SymDEq => write!(f, "⟨≡⟩"),
            SymDGt => write!(f, "⟨⇉⟩"),
            SymDLt => write!(f, "⟨⇇⟩"),
            SymDPipe => write!(f, "⟨||⟩"),
            SymDPlus => write!(f, "⟨++⟩"),
            SymDMinus => write!(f, "⟨--⟩"),
            SymDot => write!(f, "⟨`.`⟩"),
            SymEq => write!(f, "⟨=⟩"),
            SymExclaim => write!(f, "⟨!⟩"),
            SymGe => write!(f, "⟨`≥`⟩"),
            SymGt => write!(f, "⟨`>`⟩"),
            SymLBrace => write!(f, "⟨`{{`⟩"),
            SymLBracket => write!(f, "⟨`[`⟩"),
            SymLParen => write!(f, "⟨`(`⟩"),
            SymLe => write!(f, "⟨`≤`⟩"),
            SymLt => write!(f, "⟨`<`⟩"),
            SymMinus => write!(f, "⟨-⟩"),
            SymNe => write!(f, "⟨!=⟩"),
            SymPercent => write!(f, "⟨%⟩"),
            SymPipe => write!(f, "⟨|⟩"),
            SymPlus => write!(f, "⟨+⟩"),
            SymQues => write!(f, "⟨?⟩"),
            SymRBrace => write!(f, "⟨`}}`⟩"),
            SymRBracket => write!(f, "⟨`]`⟩"),
            SymRParen => write!(f, "⟨`)`⟩"),
            SymSemicolon => write!(f, "⟨`;`⟩"),
            SymSharp => write!(f, "⟨#⟩"),
            SymSlash => write!(f, "⟨`/`⟩"),
            SymTilde => write!(f, "⟨~⟩")
        }
    }
}

impl<'a> Debug for Token<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}
