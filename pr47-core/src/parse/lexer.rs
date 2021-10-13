use std::iter::Peekable;
use std::str::CharIndices;

use phf::phf_map;

use crate::syntax::token::{Token, TokenInner};

#[derive(Clone, Copy, Eq, PartialEq)]
#[repr(u8)]
pub enum LexerMode {
    LexExpr,
    LexType
}

static DEFAULT_KEYWORDS_MAP: phf::Map<&'static str, TokenInner<'static>> = phf_map! {
    "any" => TokenInner::KwdAny,
    "as" => TokenInner::KwdAs,
    "async" => TokenInner::KwdAsync,
    "auto" => TokenInner::KwdAuto,
    "await" => TokenInner::KwdAwait,
    "bool" => TokenInner::KwdBool,
    "catch" => TokenInner::KwdCatch,
    "char" => TokenInner::KwdChar,
    "const" => TokenInner::KwdConst,
    "do" => TokenInner::KwdDo,
    "else" => TokenInner::KwdElse,
    "float" => TokenInner::KwdFloat,
    "func" => TokenInner::KwdFunc,
    "if" => TokenInner::KwdIf,
    "int" => TokenInner::KwdInt,
    "object" => TokenInner::KwdObject,
    "return" => TokenInner::KwdReturn,
    "spawn" => TokenInner::KwdSpawn,
    "throw" => TokenInner::KwdThrow,
    "true" => TokenInner::KwdTrue,
    "try" => TokenInner::KwdTry,
    "type" => TokenInner::KwdType,
    "typeof" => TokenInner::KwdTypeOf,
    "var" => TokenInner::KwdVar,
    "vector" => TokenInner::KwdVector,
    "while" => TokenInner::KwdWhile
};

pub struct Lexer<'a> {
    mode: Vec<LexerMode>,
    char_indices: Peekable<CharIndices<'a>>,

    cur_ch_idx: Option<(char, usize)>,
    line: u32,
    col: u32
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        let mut ret: Self = Self {
            mode: vec![LexerMode::LexExpr],
            char_indices: source.char_indices().peekable(),

            cur_ch_idx: None,
            line: 1,
            col: 1
        };
        ret.next_char();
        ret
    }

    pub fn cur_char(&mut self) -> Option<(char, usize)> {
        self.cur_ch_idx
    }

    pub fn peek_char(&mut self) -> Option<char> {
        self.char_indices.peek().map(|(_, ch): &(usize, char)| *ch)
    }

    pub fn next_char(&mut self) {
        if let Some((idx, ch) /*: (usize, char)*/) = self.char_indices.next() {
            match ch {
                '\n' => {
                    self.line += 1;
                    self.col = 1;
                },
                '\t' => {
                    self.col += 4;
                },
                '\r' => {},
                _ => {
                    self.col += 1;
                }
            }
            self.cur_ch_idx = Some((ch, idx));
        } else {
            self.cur_ch_idx = None;
        }
    }

    pub fn push_lexer_mode(&mut self, lexer_mode: LexerMode) {
        self.mode.push(lexer_mode);
    }
    pub fn pop_lexer_mode(&mut self) {
        let _: LexerMode = self.mode.pop().unwrap();
    }
    pub fn current_mode(&self) -> LexerMode {
        *self.mode.last().unwrap()
    }
}

impl<'a> Lexer<'a> {
    pub fn next_token(&'a mut self) -> Option<Token<'a>> {
        if let Some((ch, _idx)) = self.cur_char() {
            match ch {
                'a'..='z' => {
                    todo!("红色的太阳升起在东方光芒万丈")
                },
                '0'..='9' => {
                    todo!("东风万里鲜花开放红旗像大海洋")
                },
                ch if ch.is_whitespace() => {
                    todo!("伟大的领袖英明的导师敬爱的毛主席")
                },
                ch if (ch as usize) < (' ' as usize) => {
                    todo!("革命人民心中的太阳心中的红太阳")
                },
                _ch => {
                    todo!()
                }
            }
        } else {
            None
        }
    }
}
