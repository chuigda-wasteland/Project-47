use std::cell::RefCell;
use std::iter::Peekable;
use std::str::CharIndices;

use phf::phf_map;

use crate::diag::{DiagContext, DiagMark};
use crate::diag::diag_data;
use crate::diag::location::{SourceLoc, SourceRange};
use crate::syntax::token::{Token, TokenInner};

#[derive(Clone, Copy, Eq, PartialEq)]
#[repr(u8)]
pub enum LexerMode {
    LexAttr,
    LexDecl,
    LexExpr,
    LexTopDecl,
    LexType
}

static DEFAULT_KEYWORDS_MAP: phf::Map<&'static str, TokenInner<'static>> = phf_map! {
    "any" => TokenInner::KwdAny,
    "as" => TokenInner::KwdAs,
    "auto" => TokenInner::KwdAuto,
    "await" => TokenInner::KwdAwait,
    "bool" => TokenInner::KwdBool,
    "catch" => TokenInner::KwdCatch,
    "char" => TokenInner::KwdChar,
    "const" => TokenInner::KwdConst,
    "do" => TokenInner::KwdDo,
    "else" => TokenInner::KwdElse,
    "export" => TokenInner::KwdExport,
    "float" => TokenInner::KwdFloat,
    "func" => TokenInner::KwdFunc,
    "if" => TokenInner::KwdIf,
    "import" => TokenInner::KwdImport,
    "int" => TokenInner::KwdInt,
    "object" => TokenInner::KwdObject,
    "open" => TokenInner::KwdOpen,
    "return" => TokenInner::KwdReturn,
    "spawn" => TokenInner::KwdSpawn,
    "string" => TokenInner::KwdString,
    "throw" => TokenInner::KwdThrow,
    "true" => TokenInner::KwdTrue,
    "try" => TokenInner::KwdTry,
    "type" => TokenInner::KwdType,
    "typeof" => TokenInner::KwdTypeOf,
    "using" => TokenInner::KwdUsing,
    "var" => TokenInner::KwdVar,
    "vector" => TokenInner::KwdVector,
    "void" => TokenInner::KwdVoid,
    "while" => TokenInner::KwdWhile,

    "asm" => TokenInner::RsvAsm,
    "attribute" => TokenInner::RsvAttribute,
    "ckx" => TokenInner::RsvCkx,
    "refl" => TokenInner::RsvRefl,
    "require" => TokenInner::RsvRequire
};

pub struct Lexer<'a, 'b> {
    file_id: u32,

    mode: Vec<LexerMode>,
    source: &'a str,
    char_indices: Peekable<CharIndices<'a>>,

    cur_ch_idx: Option<(char, usize)>,

    diag: &'b RefCell<DiagContext>
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
        '_' | 'A'..='Z' | 'a'..='z' | '0'..='9' => true,
        ch => !(ch.is_whitespace() || is_special(ch))
    }
}

impl<'a, 'b> Lexer<'a, 'b> {
    pub fn new(file_id: u32, source: &'a str, diag: &'b RefCell<DiagContext>) -> Self {
        let mut ret: Self = Self {
            file_id,

            mode: vec![LexerMode::LexTopDecl],
            source,
            char_indices: source.char_indices().peekable(),

            cur_ch_idx: None,

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
        let file_offset: usize = if let Some((_, idx) /*: (char, usize)*/) = self.cur_ch_idx {
            idx
        } else {
            self.source.len()
        };
        assert!(file_offset <= (u32::MAX as usize));
        SourceLoc::new(self.file_id, file_offset as u32)
    }

    pub fn next_char(&mut self) {
        if let Some((idx, ch) /*: (usize, char)*/) = self.char_indices.next() {
            if ch.is_control() && !ch.is_whitespace() {
                self.diag_unexpected_control_char(ch, SourceLoc::new(self.file_id, idx as u32));
                return self.next_char();
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
        *self.mode.last().expect("lexer does not have a mode")
    }

    pub fn prev_mode(&self) -> LexerMode {
        if self.mode.is_empty() {
            unreachable!("lexer does not have a mode")
        }

        if self.mode.len() == 1 {
            self.current_mode()
        } else {
            self.mode[self.mode.len() - 1]
        }
    }

    pub fn eoi_range(&self) -> SourceRange {
        SourceRange::new(self.file_id, self.source.len() as u32, self.source.len() as u32)
    }

    fn diag_unexpected_control_char(&mut self, ch: char, location: SourceLoc) {
        self.diag.borrow_mut()
            .diag(location, diag_data::err_unexpected_control_char_0)
            .add_mark(location.into())
            .add_arg(format!("\\{:x}", ch as u32))
            .emit();
    }
}

impl<'a, 'b> Lexer<'a, 'b> {
    pub fn next_token(&mut self) -> Token<'a> {
        if let Some((ch, _) /*: (char, usize)*/) = self.cur_char() {
            match ch {
                '/' => {
                    self.maybe_lex_comment()
                },
                'a'..='z' => {
                    self.lex_id_or_keyword()
                },
                '0'..='9' => self.lex_number_lit(),
                '\'' => self.lex_char_lit(),
                '\"' => self.lex_string_lit(),
                '`' => self.lex_raw_string_lit(),
                ch if ch.is_whitespace() => {
                    self.skip_whitespace();
                    self.next_token()
                },
                ch if is_special(ch) => {
                    self.lex_symbol()
                }
                _ => {
                    self.lex_id()
                }
            }
        } else {
            Token::new_eoi(self.eoi_range())
        }
    }

    pub fn maybe_lex_comment(&mut self) -> Token<'a> {
        let (_, offset): (char, usize) = unsafe { self.cur_char().unwrap_unchecked() };
        if let Some(ch) = self.peek_char() {
            if ch == '/' {
                self.next_char();
                self.next_char();
                while let Some(ch) = self.peek_char() {
                    if ch == '\n' {
                        self.next_char();
                        self.next_char();
                        break;
                    } else {
                        self.next_char();
                    }
                }
                self.next_token()
            } else if ch == '*' {
                self.next_char();
                self.next_char();
                while let Some(ch) = self.peek_char() {
                    if ch == '*' {
                        self.next_char();
                        if let Some('/') = self.peek_char() {
                            self.next_char();
                            self.next_char();
                            break;
                        }
                    } else {
                        self.next_char();
                    }
                }
                self.next_token()
            } else {
                self.lex_single_char_sym(
                    SourceLoc::new(self.file_id, offset as u32),
                    TokenInner::SymSlash
                )
            }
        } else {
            self.lex_single_char_sym(
                SourceLoc::new(self.file_id, offset as u32),
                TokenInner::SymSlash
            )
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
        let (_, start_idx): (char, usize) = unsafe { self.cur_char().unwrap_unchecked() };
        self.next_char();
        while let Some((ch, _) /*: (char, usize)*/) = self.cur_char() {
            if !part_of_identifier(ch) {
                break;
            }
            self.next_char();
        }

        let end_loc: SourceLoc = self.current_loc();
        let end_idx: usize = end_loc.offset as usize;
        let id: &'a str = unsafe { self.source.get_unchecked(start_idx..end_idx) };
        let range: SourceRange = SourceRange::from_loc_pair(start_loc, end_loc);

        return if self.current_mode() == LexerMode::LexAttr {
            Token::new_id(id, range)
        } else if let Some(keyword /*: TokenInner*/) = DEFAULT_KEYWORDS_MAP.get(id) {
            self.maybe_diag_reserved_keyword(keyword, id, start_loc, end_loc);
            if self.current_mode() != LexerMode::LexTopDecl &&
                *keyword == TokenInner::KwdOpen {
                Token::new_id("open", range)
            } else {
                Token::new(*keyword, range)
            }
        } else {
            self.maybe_diag_underscored_id(id, start_loc, end_loc);
            Token::new_id(id, range)
        }
    }

    pub fn lex_id(&mut self) -> Token<'a> {
        let start_loc: SourceLoc = self.current_loc();
        let (_, start_idx): (char, usize) = unsafe { self.cur_char().unwrap_unchecked() };
        self.next_char();

        while let Some((ch, idx)) = self.cur_char() {
            if part_of_identifier(ch) {
                self.next_char();
            } else {
                let end_loc: SourceLoc = self.current_loc();
                let id: &'a str = unsafe { self.source.get_unchecked(start_idx..idx) };

                return Token::new_id(id, SourceRange::from_loc_pair(start_loc, end_loc));
            }
        }

        unreachable!()
    }

    pub fn lex_symbol(&mut self) -> Token<'a> {
        let location: SourceLoc = self.current_loc();
        let (ch, _): (char, usize) = unsafe { self.cur_char().unwrap_unchecked() };

        use TokenInner::*;
        match ch {
            '&' => self.lex_maybe_consecutive(location, '&', SymAmp, SymDAmp),
            '*' => self.lex_maybe_consecutive(location, '=', SymAsterEq, SymAster),
            '\\' => self.lex_single_char_sym(location, SymBackslash),
            '^' => self.lex_maybe_consecutive(location, '^', SymDCaret, SymCaret),
            ':' => self.lex_maybe_consecutive(location, ':', SymDColon, SymColon),
            ',' => self.lex_single_char_sym(location, SymComma),
            '=' => self.lex_maybe_consecutive(location, '=', SymDEq, SymEq),
            '>' => if self.current_mode() == LexerMode::LexType {
                self.lex_single_char_sym(location, SymGt)
            } else {
                self.lex_maybe_consecutive2(location, '>', SymDGt, '=', SymGe, SymGt)
            },
            '<' => if self.current_mode() == LexerMode::LexType {
                self.lex_single_char_sym(location, SymLt)
            } else {
                self.lex_maybe_consecutive2(location, '<', SymDLt, '=', SymLe, SymLt)
            },
            '|' => self.lex_maybe_consecutive(location, '|', SymDPipe, SymPipe),
            '+' => self.lex_maybe_consecutive(location, '=', SymPlusEq, SymPlus),
            '-' => self.lex_maybe_consecutive(location, '=', SymMinusEq, SymMinus),
            '.' => self.lex_single_char_sym(location, SymDot),
            '!' => self.lex_maybe_consecutive(location, '=', SymNe, SymExclaim),
            '#' => self.lex_single_char_sym(location, SymHash),
            '{' => self.lex_single_char_sym(location, SymLBrace),
            '[' => self.lex_single_char_sym(location, SymLBracket),
            '(' => self.lex_single_char_sym(location, SymLParen),
            '%' => self.lex_maybe_consecutive(location, '=', SymPercentEq, SymPercent),
            '?' => self.lex_single_char_sym(location, SymQues),
            '}' => self.lex_single_char_sym(location, SymRBrace),
            ']' => self.lex_single_char_sym(location, SymRBracket),
            ')' => self.lex_single_char_sym(location, SymRParen),
            ';' => self.lex_single_char_sym(location, SymSemicolon),
            '/' => self.lex_maybe_consecutive(location, '=', SymSlashEq, SymSlash),
            '~' => self.lex_single_char_sym(location, SymTilde),
            '@' => self.lex_reserved_sym(location, RsymAt, '@'),
            '$' => self.lex_reserved_sym(location, RsymDollar, '$'),
            _ => unreachable!()
        }
    }

    pub fn lex_number_lit(&mut self) -> Token<'a> {
        let start_loc: SourceLoc = self.current_loc();
        let (ch, _) = self.cur_char().unwrap();
        if ch == '0' {
            if let Some(ch) = self.peek_char() {
                if ch == 'x' || ch == 'X' {
                    self.next_char();
                    self.next_char();
                    return self.lex_num_radix_lit(start_loc, 16);
                } else if ch == 'o' || ch == 'O' {
                    self.next_char();
                    self.next_char();
                    return self.lex_num_radix_lit(start_loc, 8);
                } else if ch == 'b' || ch == 'B' {
                    self.next_char();
                    self.next_char();
                    return self.lex_num_radix_lit(start_loc, 2);
                } else if ch.is_ascii_digit() {
                    self.diag.borrow_mut()
                        .diag(self.current_loc(),
                              diag_data::err_bad_num_literal_hex_oct_bin)
                        .add_mark(self.current_loc().into())
                        .emit();
                }
            }
        }

        let mut integral_part: String = String::new();
        while let Some((ch, _)) = self.cur_char() {
            if ch.is_ascii_digit() {
                integral_part.push(ch);
                self.next_char();
            } else {
                break;
            }
        }

        if let Some((ch, _)) = self.cur_char() {
            if ch == '.' || ch == 'e' || ch == 'E' {
                return self.lex_float_lit(start_loc, integral_part);
            }
        }

        let end_loc: SourceLoc = self.current_loc();
        Token::new_lit_int(
            integral_part.parse::<u64>().unwrap(),
            SourceRange::from_loc_pair(start_loc, end_loc)
        )
    }

    fn lex_num_radix_lit(&mut self, start_loc: SourceLoc, radix: u32) -> Token<'a> {
        let mut hex_lit: String = String::new();
        while let Some((ch, _)) = self.cur_char() {
            if ch.is_digit(radix) {
                hex_lit.push(ch);
                self.next_char();
            } else {
                break;
            }
        }

        let end_loc: SourceLoc = self.current_loc();

        if hex_lit.is_empty() {
            self.diag.borrow_mut()
                .diag(end_loc, diag_data::err_empty_literal)
                .add_mark(end_loc.into())
                .emit();
            return Token::new_lit_int(0, SourceRange::from_loc_pair(start_loc, end_loc));
        }

        Token::new_lit_int(
            u64::from_str_radix(&hex_lit, radix).unwrap(),
            SourceRange::from_loc_pair(start_loc, end_loc)
        )
    }

    fn lex_float_lit(&mut self, start_loc: SourceLoc, integral_part: String) -> Token<'a> {
        let mut fractional_part: String = String::new();
        if let Some((ch, _)) = self.cur_char() {
            if ch == '.' {
                self.next_char();
                while let Some((ch, _)) = self.cur_char() {
                    if ch.is_ascii_digit() {
                        fractional_part.push(ch);
                        self.next_char();
                    } else {
                        break;
                    }
                }
            }
        }
        let fractional_part: u64 = fractional_part.parse::<u64>().unwrap_or(0);

        let mut exponent: String = String::new();
        if let Some((ch, _)) = self.cur_char() {
            if ch == 'e' || ch == 'E' {
                self.next_char();
                if let Some((ch, _)) = self.cur_char() {
                    if ch == '-' || ch == '+' {
                        exponent.push(ch);
                        self.next_char();
                    }
                }
                while let Some((ch, _)) = self.cur_char() {
                    if ch.is_ascii_digit() {
                        exponent.push(ch);
                        self.next_char();
                    } else {
                        break;
                    }
                }
            }
        }
        let exponent: u64 = if exponent.is_empty() || exponent == "+" || exponent == "-" {
            self.diag.borrow_mut()
                .diag(self.current_loc(),
                      diag_data::err_empty_float_exponent)
                .add_mark(self.current_loc().into())
                .emit();
            0
        } else {
            exponent.parse::<u64>().unwrap()
        };

        let end_loc: SourceLoc = self.current_loc();
        let float_lit: String = format!("{}.{}e{}", integral_part, fractional_part, exponent);

        Token::new_lit_float(
            float_lit.parse::<f64>().unwrap(),
            SourceRange::from_loc_pair(start_loc, end_loc)
        )
    }

    pub fn lex_char_lit(&mut self) -> Token<'a> {
        todo!("인민의 운명을 한몸에 안고")
    }

    pub fn lex_string_lit(&mut self) -> Token<'a> {
        let start_loc: SourceLoc = self.current_loc();
        self.next_char();

        while let Some((ch, _)) = self.cur_char() {
            if ch == '"' {
                let string_end_loc: SourceLoc = self.current_loc();
                self.next_char();

                let str: &'a str = unsafe { self.slice_source(start_loc.offset, string_end_loc.offset) };
                return Token::new_lit_str(str, SourceRange::from_loc_pair(start_loc, self.current_loc()));
            }

            if ch == '\\' {
                self.next_char();
                if let Some((ch, _)) = self.cur_char() {
                    match ch {
                        'n' | 't' | 'r' | 'f' | 'v' | '"' | '\'' | '\\' => self.next_char(),
                        _ => self.diag.borrow_mut()
                            .diag(self.current_loc(), diag_data::err_bad_escape)
                            .add_mark(self.current_loc().into())
                            .add_arg(ch)
                            .emit(),
                    }
                }
            } else {
                self.next_char();
            }
        }

        let string_end_loc: SourceLoc = self.current_loc();
        self.diag.borrow_mut()
            .diag(self.current_loc(), diag_data::err_unclosed_string)
            .add_mark(string_end_loc.into())
            .emit();
        let str: &'a str = unsafe { self.slice_source(start_loc.offset, string_end_loc.offset) };

        Token::new_lit_str(str, SourceRange::from_loc_pair(start_loc, string_end_loc))
    }

    pub fn lex_raw_string_lit(&mut self) -> Token<'a> {
        todo!("모두다 꽃펴주실 분")
    }

    fn lex_single_char_sym(&mut self, location: SourceLoc, token: TokenInner<'a>) -> Token<'a> {
        self.next_char();
        Token::new(token, SourceRange::from(location))
    }

    fn lex_maybe_consecutive(
        &mut self,
        location: SourceLoc,
        ch: char,
        consecutive: TokenInner<'a>,
        otherwise: TokenInner<'a>
    ) -> Token<'a> {
        self.next_char();

        if let Some((got_ch, _) /*: (char, usize)*/) = self.cur_char() {
            if got_ch == ch {
                self.next_char();
                return Token::new(consecutive, SourceRange::from(location))
            }
        }

        Token::new(otherwise, SourceRange::from(location))
    }

    fn lex_maybe_consecutive2(
        &mut self,
        location: SourceLoc,
        ch1: char,
        consecutive1: TokenInner<'a>,
        ch2: char,
        consecutive2: TokenInner<'a>,
        otherwise: TokenInner<'a>
    ) -> Token<'a> {
        self.next_char();

        if let Some((got_ch, _) /*: (char, usize)*/) = self.cur_char() {
            if got_ch == ch1 {
                self.next_char();
                return Token::new(consecutive1, SourceRange::from(location))
            } else if got_ch == ch2 {
                self.next_char();
                return Token::new(consecutive2, SourceRange::from(location))
            }
        }

        Token::new(otherwise, SourceRange::from(location))
    }

    fn lex_reserved_sym(
        &mut self,
        location: SourceLoc,
        token: TokenInner<'a>,
        ch: char
    ) -> Token<'a> {
        self.diag.borrow_mut()
            .diag(location, diag_data::err_reserved_symbol_0)
            .add_mark(DiagMark::from(location))
            .add_arg(ch.to_string())
            .emit();
        self.next_char();
        Token::new(token, SourceRange::from(location))
    }

    unsafe fn slice_source(&self, start_offset: u32, end_offset: u32) -> &'a str {
        self.source.get_unchecked((start_offset as usize)..(end_offset as usize))
    }

    fn maybe_diag_reserved_keyword(
        &mut self,
        keyword: &TokenInner,
        id: &str,
        start_loc: SourceLoc,
        end_loc: SourceLoc
    ) {
        if keyword.is_reserved() {
            self.diag.borrow_mut()
                .diag(start_loc, diag_data::err_reserved_identifier_0)
                .add_mark(
                    DiagMark::from(SourceRange::from_loc_pair(start_loc, end_loc))
                        .add_comment("reserved identifier")
                )
                .add_arg(id)
                .emit();
        }
    }

    fn maybe_diag_underscored_id(&mut self, id: &str, start_loc: SourceLoc, end_loc: SourceLoc) {
        if id.starts_with('_') {
            self.diag.borrow_mut()
                .diag(start_loc, diag_data::warn_underscored_id_reserved)
                .add_mark(DiagMark::from(SourceRange::from_loc_pair(start_loc, end_loc)))
                .emit();
        }
    }
}
