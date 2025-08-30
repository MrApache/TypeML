use logos::{Lexer, Logos, Source, Span};
use tower_lsp::lsp_types::{Position as LspPosition, Range, SemanticToken};

pub const KEYWORD_TOKEN: u32 = 0;
pub const PARAMETER_TOKEN: u32 = 1;
pub const STRING_TOKEN: u32 = 2;
pub const TYPE_TOKEN: u32 = 3;
pub const OPERATOR_TOKEN: u32 = 4;
pub const NUMBER: u32 = 5;
pub const COMMENT: u32 = 6;
pub const MACRO_TOKEN: u32 = 7;
pub const FUNCTION: u32 = 8;

#[macro_export]
macro_rules! push_and_break {
    ($tokens:expr, $kind:expr, $lex:expr) => {{
        Token::push_with_advance($tokens, $kind, $lex);
        break;
    }};
}

#[macro_export]
macro_rules! new_line_and_break {
    ($lex:expr) => {{
        $lex.extras.new_line();
        break;
    }};
}

#[derive(Default, Clone)]
pub struct Position {
    previous_token_line: u32,
    current_line: u32,

    previous_token_start_column: u32,
    current_column: u32,

    line: u32,
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

        self.line += 1;
    }

    pub const fn advance(&mut self, length: u32) {
        self.current_column += length;
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Token<T> {
    kind: T,
    span: Span,
    delta_line: u32,
    delta_start: u32,
    start: LspPosition,
    end: LspPosition,
}

impl<T> Token<T> {
    pub fn new<'source, Token>(kind: T, lexer: &mut Lexer<'source, Token>) -> Self
    where
        Token: Logos<'source, Extras = Position>,
    {
        let delta_line = lexer.extras.get_delta_line();
        let delta_start = lexer.extras.get_delta_start();
        Self::new_custom(kind, lexer, lexer.span(), delta_line, delta_start)
    }

    pub fn new_with_span<'source, Token>(
        kind: T,
        lexer: &mut Lexer<'source, Token>,
        span: Span,
    ) -> Self
    where
        Token: Logos<'source, Extras = Position>,
    {
        let delta_line = lexer.extras.get_delta_line();
        let delta_start = lexer.extras.get_delta_start();
        Self::new_custom(kind, lexer, span, delta_line, delta_start)
    }

    pub fn new_custom<'source, Token>(
        kind: T,
        lexer: &mut Lexer<'source, Token>,
        span: Span,
        delta_line: u32,
        delta_start: u32,
    ) -> Self
    where
        Token: Logos<'source, Extras = Position>,
    {
        Self {
            kind,
            delta_line,
            delta_start,
            start: LspPosition {
                line: lexer.extras.line,
                character: lexer.extras.current_column,
            },
            end: LspPosition {
                line: lexer.extras.line,
                character: lexer.extras.current_column + span.len() as u32,
            },
            span,
        }
    }

    pub const fn kind(&self) -> &T {
        &self.kind
    }

    pub fn take_kind(self) -> T {
        self.kind
    }

    pub fn span(&self) -> Span {
        self.span.clone()
    }

    pub fn slice<'a>(&self, str: &'a str) -> &'a str {
        str.slice(self.span()).unwrap()
    }
 
    pub const fn delta_line(&self) -> u32 {
        self.delta_line
    }

    pub const fn delta_start(&self) -> u32 {
        self.delta_start
    }

    pub const fn start(&self) -> LspPosition {
        self.start
    }

    pub const fn end(&self) -> LspPosition {
        self.end
    }

    pub const fn range(&self) -> Range {
        Range {
            start: self.start,
            end: self.end,
        }
    }

    pub fn length(&self) -> u32 {
        self.span().len() as u32
    }

    pub fn to_semantic_token(&self, token_type: u32) -> SemanticToken {
        SemanticToken {
            delta_line: self.delta_line(),
            delta_start: self.delta_start(),
            length: self.length(),
            token_type,
            token_modifiers_bitset: 0,
        }
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
