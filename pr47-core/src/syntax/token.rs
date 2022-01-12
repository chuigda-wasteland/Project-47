use std::fmt::{Display, Formatter};
use std::mem::discriminant;

use smallvec::alloc::fmt::Debug;
use xjbutil::display2::Display2;

use crate::diag::location::SourceRange;

#[derive(Clone, Copy)]
#[cfg_attr(any(debug_assertions, test), derive(Debug))]
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
    KwdExport,
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
    KwdString,
    KwdThrow,
    KwdTrue,
    KwdTry,
    KwdType,
    KwdTypeOf,
    KwdVar,
    KwdVector,
    KwdVoid,
    KwdWhile,

    LitChar(char),
    LitFloat(f64),
    LitInt(u64),
    LitSignedInt(i64),
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
    SymAsterEq,
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
    SymHash,
    SymLBrace,
    SymLBracket,
    SymLParen,
    SymLe,
    SymLt,
    SymMinus,
    SymMinusEq,
    SymNe,
    SymPercent,
    SymPercentEq,
    SymPipe,
    SymPlus,
    SymPlusEq,
    SymQues,
    SymQuesColon,
    SymRBrace,
    SymRBracket,
    SymRParen,
    SymSemicolon,
    SymSlash,
    SymSlashEq,
    SymTilde,

    EndOfInput
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

impl<'a> PartialEq for TokenInner<'a> {
    fn eq(&self, other: &Self) -> bool {
        discriminant(self) == discriminant(other)
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

    pub fn new_eoi(range: SourceRange) -> Self {
        Self::new(TokenInner::EndOfInput, range)
    }

    pub fn new_lit_int(lit: u64, range: SourceRange) -> Self {
        Self::new(TokenInner::LitInt(lit), range)
    }

    pub fn new_lit_signed_int(lit: i64, range: SourceRange) -> Self {
        Self::new(TokenInner::LitSignedInt(lit), range)
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

    pub fn is_eoi(&self) -> bool {
        self.token_inner == TokenInner::EndOfInput
    }

    pub fn get_str_value(&self) -> &'a str {
        match self.token_inner {
            TokenInner::Ident(id) => id,
            TokenInner::LitStr(s) => s,
            _ => panic!("this token should be either identifier, or string literal")
        }
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
            KwdExport => write!(f, "⟨export⟩"),
            KwdFalse => write!(f, "⟨false⟩"),
            KwdFloat => write!(f, "⟨float⟩"),
            KwdFunc => write!(f, "⟨func⟩"),
            KwdIf => write!(f, "⟨if⟩"),
            KwdImport => write!(f, "⟨import⟩"),
            KwdInt => write!(f, "⟨int⟩"),
            KwdObject => write!(f, "⟨object⟩"),
            KwdOpen => write!(f, "⟨open⟩"),
            KwdReturn => write!(f, "⟨return⟩"),
            KwdString => write!(f, "⟨string⟩"),
            KwdSpawn => write!(f, "⟨spawn⟩"),
            KwdThrow => write!(f, "⟨throw⟩"),
            KwdTrue => write!(f, "⟨true⟩"),
            KwdTry => write!(f, "⟨try⟩"),
            KwdType => write!(f, "⟨type⟩"),
            KwdTypeOf => write!(f, "⟨typeof⟩"),
            KwdVar => write!(f, "⟨var⟩"),
            KwdVoid => write!(f, "⟨void⟩"),
            KwdVector => write!(f, "⟨vector⟩"),
            KwdWhile => write!(f, "⟨while⟩"),

            LitChar(ch) => write!(f, "⟨char, '{}'⟩", ch),
            LitFloat(num) => write!(f, "⟨num, {}f⟩", num),
            LitInt(num) => write!(f, "⟨num, {}i⟩", num),
            LitSignedInt(num) => write!(f, "⟨num, {}i⟩", num),
            LitStr(str) => write!(f, "⟨str, \"{}\"⟩", str),

            RsvAsm => write!(f, "⟨asm⟩"),
            RsvAttribute => write!(f, "⟨attribute⟩"),
            RsvCkx => write!(f, "⟨ckx⟩"),
            RsvRefl => write!(f, "⟨refl⟩"),
            RsvRequire => write!(f, "⟨require⟩"),
            RsymAt => write!(f, "⟨@⟩"),
            RsymDollar => write!(f, "⟨$⟩"),

            SymAmp => write!(f, "⟨&⟩"),
            SymAster => write!(f, "⟨*⟩"),
            SymAsterEq => write!(f, "⟨*=⟩"),
            SymBackslash => write!(f, "⟨'\\'⟩"),
            SymCaret => write!(f, "⟨^⟩"),
            SymColon => write!(f, "⟨':'⟩"),
            SymComma => write!(f, "⟨','⟩"),
            SymDAmp => write!(f, "⟨&&⟩"),
            SymDCaret => write!(f, "⟨^^⟩"),
            SymDColon => write!(f, "⟨::⟩"),
            SymDEq => write!(f, "⟨≡⟩"),
            SymDGt => write!(f, "⟨⇉⟩"),
            SymDLt => write!(f, "⟨⇇⟩"),
            SymDPipe => write!(f, "⟨||⟩"),
            SymDot => write!(f, "⟨'.'⟩"),
            SymEq => write!(f, "⟨=⟩"),
            SymExclaim => write!(f, "⟨!⟩"),
            SymGe => write!(f, "⟨'≥'⟩"),
            SymGt => write!(f, "⟨'>'⟩"),
            SymHash => write!(f, "⟨#⟩"),
            SymLBrace => write!(f, "⟨'{{'⟩"),
            SymLBracket => write!(f, "⟨'['⟩"),
            SymLParen => write!(f, "⟨'('⟩"),
            SymLe => write!(f, "⟨'≤'⟩"),
            SymLt => write!(f, "⟨'<'⟩"),
            SymMinus => write!(f, "⟨-⟩"),
            SymMinusEq => write!(f, "⟨-=⟩"),
            SymNe => write!(f, "⟨!=⟩"),
            SymPercent => write!(f, "⟨%⟩"),
            SymPercentEq => write!(f, "⟨%=⟩"),
            SymPipe => write!(f, "⟨|⟩"),
            SymPlus => write!(f, "⟨+⟩"),
            SymPlusEq => write!(f, "⟨+=⟩"),
            SymQues => write!(f, "⟨?⟩"),
            SymQuesColon => write!(f, "⟨?:⟩"),
            SymRBrace => write!(f, "⟨'}}'⟩"),
            SymRBracket => write!(f, "⟨']'⟩"),
            SymRParen => write!(f, "⟨')'⟩"),
            SymSemicolon => write!(f, "⟨';'⟩"),
            SymSlash => write!(f, "⟨'/'⟩"),
            SymSlashEq => write!(f, "⟨/=⟩"),
            SymTilde => write!(f, "⟨'~'⟩"),

            EndOfInput => write!(f, "♦")
        }
    }
}

impl<'a> Debug for Token<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl<'a> Display2 for TokenInner<'a> {
    fn fmt2(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenInner::Ident(ident) => if ident.is_empty() {
                write!(fmt, "identifier")
            } else {
                write!(fmt, "identifier \"{}\"", ident)
            },

            TokenInner::KwdAny => write!(fmt, "'any'"),
            TokenInner::KwdAs => write!(fmt, "'as'"),
            TokenInner::KwdAuto => write!(fmt, "'auto'"),
            TokenInner::KwdAwait => write!(fmt, "'await'"),
            TokenInner::KwdBool => write!(fmt, "'bool'"),
            TokenInner::KwdCatch => write!(fmt, "'catch'"),
            TokenInner::KwdChar => write!(fmt, "'char'"),
            TokenInner::KwdConst => write!(fmt, "'const'"),
            TokenInner::KwdDo => write!(fmt, "'do'"),
            TokenInner::KwdElse => write!(fmt, "'else'"),
            TokenInner::KwdExport => write!(fmt, "'export'"),
            TokenInner::KwdFalse => write!(fmt, "'false'"),
            TokenInner::KwdFloat => write!(fmt, "'float'"),
            TokenInner::KwdFunc => write!(fmt, "'func'"),
            TokenInner::KwdIf => write!(fmt, "'if'"),
            TokenInner::KwdImport => write!(fmt, "'import'"),
            TokenInner::KwdInt => write!(fmt, "'int'"),
            TokenInner::KwdObject => write!(fmt, "'object'"),
            TokenInner::KwdOpen => write!(fmt, "'open'"),
            TokenInner::KwdReturn => write!(fmt, "'any'"),
            TokenInner::KwdSpawn => write!(fmt, "'spawn'"),
            TokenInner::KwdString => write!(fmt, "'string'"),
            TokenInner::KwdThrow => write!(fmt, "'throw'"),
            TokenInner::KwdTrue => write!(fmt, "'true'"),
            TokenInner::KwdTry => write!(fmt, "'try'"),
            TokenInner::KwdType => write!(fmt, "'type'"),
            TokenInner::KwdTypeOf => write!(fmt, "'typeof'"),
            TokenInner::KwdVar => write!(fmt, "'var'"),
            TokenInner::KwdVector => write!(fmt, "'vector'"),
            TokenInner::KwdVoid => write!(fmt, "'void'"),
            TokenInner::KwdWhile => write!(fmt, "'write'"),

            TokenInner::LitChar(_) => write!(fmt, "char literal"),
            TokenInner::LitFloat(_) => write!(fmt, "float literal"),
            TokenInner::LitInt(_) => write!(fmt, "integer literal"),
            TokenInner::LitSignedInt(_) => write!(fmt, "integer literal"),
            TokenInner::LitStr(_) => write!(fmt, "string literal"),

            TokenInner::RsvAsm => write!(fmt, "'asm'"),
            TokenInner::RsvAttribute => write!(fmt, "'attribute'"),
            TokenInner::RsvCkx => write!(fmt, "'ckx'"),
            TokenInner::RsvRefl => write!(fmt, "'refl'"),
            TokenInner::RsvRequire => write!(fmt, "'require'"),

            TokenInner::RsymAt => write!(fmt, "'@'"),
            TokenInner::RsymDollar => write!(fmt, "'$'"),
            TokenInner::SymAmp => write!(fmt, "'&'"),
            TokenInner::SymAster => write!(fmt, "'*'"),
            TokenInner::SymAsterEq => write!(fmt, "'*='"),
            TokenInner::SymBackslash => write!(fmt, "'\\'"),
            TokenInner::SymCaret => write!(fmt, "'^'"),
            TokenInner::SymColon => write!(fmt, "':'"),
            TokenInner::SymComma => write!(fmt, "','"),
            TokenInner::SymDAmp => write!(fmt, "'&&'"),
            TokenInner::SymDCaret => write!(fmt, "'^^'"),
            TokenInner::SymDColon => write!(fmt, "'::'"),
            TokenInner::SymDEq => write!(fmt, "'=='"),
            TokenInner::SymDGt => write!(fmt, "'>>'"),
            TokenInner::SymDLt => write!(fmt, "'<<'"),
            TokenInner::SymDPipe => write!(fmt, "'||'"),
            TokenInner::SymDot => write!(fmt, "'.'"),
            TokenInner::SymEq => write!(fmt, "'='"),
            TokenInner::SymExclaim => write!(fmt, "'!'"),
            TokenInner::SymGe => write!(fmt, "'>='"),
            TokenInner::SymGt => write!(fmt, "'>'"),
            TokenInner::SymHash => write!(fmt, "'#'"),
            TokenInner::SymLBrace => write!(fmt, "'{{'"),
            TokenInner::SymLBracket => write!(fmt, "'['"),
            TokenInner::SymLParen => write!(fmt, "'('"),
            TokenInner::SymLe => write!(fmt, "'<='"),
            TokenInner::SymLt => write!(fmt, "'<'"),
            TokenInner::SymMinus => write!(fmt, "'-'"),
            TokenInner::SymMinusEq => write!(fmt, "'-='"),
            TokenInner::SymNe => write!(fmt, "'!='"),
            TokenInner::SymPercent => write!(fmt, "'%'"),
            TokenInner::SymPercentEq => write!(fmt, "'%='"),
            TokenInner::SymPipe => write!(fmt, "'|'"),
            TokenInner::SymPlus => write!(fmt, "'+'"),
            TokenInner::SymPlusEq => write!(fmt, "'+='"),
            TokenInner::SymQues => write!(fmt, "'?'"),
            TokenInner::SymQuesColon => write!(fmt, "'?:'"),
            TokenInner::SymRBrace => write!(fmt, "'}}'"),
            TokenInner::SymRBracket => write!(fmt, "']'"),
            TokenInner::SymRParen => write!(fmt, "')'"),
            TokenInner::SymSemicolon => write!(fmt, "';'"),
            TokenInner::SymSlash => write!(fmt, "'/'"),
            TokenInner::SymSlashEq => write!(fmt, "'/='"),
            TokenInner::SymTilde => write!(fmt, "'~'"),
            TokenInner::EndOfInput => write!(fmt, "end of input")
        }
    }
}
