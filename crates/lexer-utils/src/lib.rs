use logos::{Lexer, Logos, Span};

pub const KEYWORD: u32 = 0;
pub const PARAMETER: u32 = 1;
pub const STRING: u32 = 2;
pub const TYPE: u32 = 3;
pub const OPERATOR: u32 = 4;
pub const NUMBER: u32 = 5;
pub const COMMENT: u32 = 6;
pub const MACRO: u32 = 7;
pub const FUNCTION: u32 = 8;

#[macro_export]
macro_rules! push_and_break {
    ($tokens:expr, $kind:expr, $lex:expr) => {{
        Token::push_with_advance($tokens, $kind, $lex);
        break;
    }};
}

pub trait TokenType {
    fn get_token_type(&self) -> u32;
}

#[derive(Default, Clone)]
pub struct Position {
    previous_token_line: u32,
    current_line: u32,

    previous_token_start_column: u32,
    pub current_column: u32,
}

impl Position {
    pub const fn get_delta_line(&mut self) -> u32 {
        let delta = self.current_line - self.previous_token_line;
        self.previous_token_line = self.current_line;
        delta
    }

    pub const fn get_delta_start(&mut self) -> u32 {
        let delta = self.current_column - self.previous_token_start_column;
        self.previous_token_start_column = self.current_column;
        delta
    }

    pub const fn new_line(&mut self) {
        self.current_column = 0;
        self.current_line += 1;
        self.previous_token_start_column = 0;
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Token<T> {
    pub kind: T,
    pub span: Span,
    pub delta_line: u32, 
    pub delta_start: u32,
}

impl<T> Token<T> {
    pub fn new<'source, Token>(kind: T, lexer: &mut Lexer<'source, Token>) -> Self 
    where
        Token: Logos<'source, Extras = Position>,
    {
        Self {
            kind,
            span: lexer.span(),
            delta_line: lexer.extras.get_delta_line(),
            delta_start: lexer.extras.get_delta_start(),
        }
    }

    pub const fn kind(&self) -> &T {
        &self.kind
    }

    pub fn span(&self) -> Span {
        self.span.clone()
    }

    pub const fn delta_line(&self) -> u32 {
        self.delta_line
    }

    pub const fn delta_start(&self) -> u32 {
        self.delta_start
    }

    pub fn length(&self) -> u32 {
        self.span().len() as u32
    }

    pub fn push_with_advance<'s, Tok: Logos<'s, Extras = Position>>(
        tokens: &mut Vec<Self>,
        kind: impl Into<T>,
        lex: &mut Lexer<'s, Tok>,
    ) {
        tokens.push(Token::new(kind.into(), lex));
        lex.extras.current_column += lex.span().len() as u32;
    }
}
