use std::iter::Peekable;
use std::str::CharIndices;

use phf::phf_map;
use unchecked_unwrap::UncheckedUnwrap;

use crate::syntax::token::{Token, TokenInner};
use crate::util::diag::{DiagContext, messages, DiagMark};
use crate::util::location::SourceLoc;

#[cfg(feature = "compiler-pretty-diag")] use unicode_width::UnicodeWidthChar;

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
    file: &'a str,

    mode: Vec<LexerMode>,
    source: &'a str,
    char_indices: Peekable<CharIndices<'a>>,

    cur_ch_idx: Option<(char, usize)>,
    line: u32,
    col: u32,

    diag: &'a mut DiagContext<'a>
}

pub fn is_special(ch: char) -> bool {
    match ch {
        '@' | '$' | '&' | '*' | '\\' | '^' | ':' | ',' | '=' | '>' | '<' | '|' | '.' | '!' |
        '{' | '[' | '(' | '-' | '%' | '+' | '?' | '}' | ']' | ')' | ';' | '#' | '/' | '~' => true,
        _ => false
    }
}

pub fn part_of_identifier(ch: char) -> bool {
    match ch {
        '_' | 'A'..='Z' | 'a'..='z' | '0'..='9' | '!' | '?' => true,
        ch => !(ch.is_whitespace() || is_special(ch))
    }
}

impl<'a> Lexer<'a> {
    pub fn new(file: &'a str, source: &'a str, diag: &'a mut DiagContext<'a>) -> Self {
        let mut ret: Self = Self {
            file,

            mode: vec![LexerMode::LexExpr],
            source,
            char_indices: source.char_indices().peekable(),

            cur_ch_idx: None,
            line: 1,
            col: 1,

            diag
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

    pub fn current_loc(&mut self) -> SourceLoc {
        SourceLoc::new(self.line, self.col)
    }

    pub fn next_char(&mut self) {
        if let Some((idx, ch) /*: (usize, char)*/) = self.char_indices.next() {
            match ch {
                '\n' => {
                    self.line += 1;
                    self.col = 0;
                },
                '\t' => self.col += 4,
                '\r' => {},
                ' ' => self.col += 1,
                ch => {
                    let location: SourceLoc = self.current_loc();

                    self.maybe_diag_non_ascii_whitespace(ch, location);

                    #[cfg(feature = "compiler-pretty-diag")]
                    if let Some(width /*: usize*/) = ch.width() {
                        self.col += width as u32;
                    } else {
                        self.diag_unexpected_control_char(ch, location);
                        return self.next_char();
                    }

                    #[cfg(not(feature = "compiler-pretty-diag"))]
                    if ch.is_control() {
                        self.diag_unexpected_control_char(ch, location);
                        return self.next_char();
                    }
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

    fn maybe_diag_non_ascii_whitespace(&mut self, ch: char, location: SourceLoc) {
        if ch.is_whitespace() && !ch.is_ascii_whitespace() {
            self.diag.diag(self.file, messages::warn_space_character_0_ignored)
                .add_location(location)
                .add_mark(DiagMark::from(location).add_comment("non_ascii whitespace"))
                .add_arg(format!("\\{:x}", ch as u32))
                .build();
        }
    }

    fn diag_unexpected_control_char(&mut self, ch: char, location: SourceLoc) {
        self.diag.diag(self.file, messages::err_unexpected_control_char_0)
            .add_location(location)
            .add_mark(location.into())
            .add_arg(format!("\\{:x}", ch as u32))
            .build();
    }
}

impl<'a> Lexer<'a> {
    pub fn next_token(&mut self) -> Option<Token<'a>> {
        if let Some((ch, _) /*: (char, usize)*/) = self.cur_char() {
            match ch {
                'a'..='z' => {
                    Some(self.lex_id_or_keyword())
                },
                '0'..='9' => {
                    todo!("Nhu co bac ho trong ngay vui dai thang")
                },
                ch if ch.is_whitespace() => {
                    self.skip_whitespace();
                    return self.next_token();
                },
                ch if is_special(ch) => {
                    todo!("Loi bac nay da thanh chien thang huy hoang")
                }
                _ => {
                    Some(self.lex_id())
                }
            }
        } else {
            None
        }
    }

    pub fn skip_whitespace(&mut self) {
        while let Some((ch, _) /*: (char, usize)*/) = self.cur_char() {
            if ch.is_whitespace() {
                self.next_char()
            } else {
                break;
            }
        }
    }

    pub fn lex_id_or_keyword(&mut self) -> Token<'a> {
        let start_loc: SourceLoc = self.current_loc();
        let (_, start_idx): (char, usize) = unsafe { self.cur_char().unchecked_unwrap() };
        self.next_char();

        let mut maybe_end_loc: SourceLoc = self.current_loc();
        while let Some((ch, idx) /*: (char, usize)*/) = self.cur_char() {
            if part_of_identifier(ch) {
                maybe_end_loc = self.current_loc();
                self.next_char();
            } else {
                let id: &'a str = unsafe { self.source.get_unchecked(start_idx..idx) };
                return if let Some(keyword /*: TokenInner*/) = DEFAULT_KEYWORDS_MAP.get(id) {
                    self.maybe_diag_reserved_keyword(keyword, id, start_loc, maybe_end_loc);
                    Token::new(*keyword, start_loc, maybe_end_loc)
                } else {
                    self.maybe_diag_underscored_id(id, start_loc, maybe_end_loc);
                    Token::new_id(id, start_loc, maybe_end_loc)
                }
            }
        }

        unreachable!()
    }

    pub fn lex_id(&mut self) -> Token<'a> {
        let start_loc: SourceLoc = self.current_loc();
        let (_, start_idx): (char, usize) = unsafe { self.cur_char().unchecked_unwrap() };
        self.next_char();

        let mut maybe_end_loc: SourceLoc = self.current_loc();
        while let Some((ch, idx)) = self.cur_char() {
            if part_of_identifier(ch) {
                maybe_end_loc = self.current_loc();
                self.next_char();
            } else {
                let id: &'a str = unsafe { self.source.get_unchecked(start_idx..idx) };

                return Token::new_id(id, start_loc, maybe_end_loc);
            }
        }

        unreachable!()
    }

    fn maybe_diag_reserved_keyword(
        &mut self,
        keyword: &TokenInner,
        id: &str,
        start_loc:
        SourceLoc,
        end_loc: SourceLoc
    ) {
        if keyword.is_reserved() {
            self.diag.diag(self.file, messages::err_reserved_identifier_0)
                .add_location(start_loc)
                .add_mark(DiagMark::new(
                    start_loc.line, start_loc.col, end_loc.col
                ).add_comment("reserved identifier"))
                .add_arg(id)
                .build();
        }
    }

    fn maybe_diag_underscored_id(&mut self, id: &str, start_loc: SourceLoc, end_loc: SourceLoc) {
        if id.starts_with('_') {
            self.diag.diag(self.file, messages::warn_underscored_id_reserved)
                .add_location(start_loc)
                .add_mark(DiagMark::new(
                    start_loc.line, start_loc.col, end_loc.col
                ))
                .build();
        }
    }
}
