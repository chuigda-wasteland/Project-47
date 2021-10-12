#[derive(Clone, Copy, Eq, PartialEq)]
#[repr(u8)]
pub enum LexerMode {
    LexCommon,
    LexType,
    LexExpr
}

pub struct Lexer<'a> {
    #[allow(dead_code)]
    source: &'a str
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self { source }
    }

    pub fn push_lexer_mode(&mut self, _lexer_mode: LexerMode) { todo!() }
    pub fn pop_lexer_mode(&mut self, _lexer_mode: LexerMode) { todo!() }
    pub fn current_mode(&self) -> LexerMode { todo!() }
}
