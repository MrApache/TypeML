use crate::{Error, NamedStatement, SchemaStatement, StatementTokens};
use lexer_utils::{push_and_break, unwrap_or_continue, Position, Token};
use logos::{Lexer, Logos};
use std::fmt::Display;

#[derive(Logos, Debug, PartialEq, Eq, Clone)]
#[logos(extras = Position)]
#[logos(error(Error, Error::from_lexer))]
pub enum GroupDefinitionToken {
    Keyword,

    #[regex("[a-zA-Z_][a-zA-Z0-9_]*")]
    Identifier,

    #[token("|")]
    Pipe,

    #[regex(r"\[|\?|\*|\+", quantifier_callback)]
    Quantifier(Vec<Token<QuantifierToken>>),

    #[token(";")]
    Semicolon,

    #[token("\n")]
    NewLine,

    #[regex(r"[ \t\r]+")]
    Whitespace,

    SyntaxError,
}

impl Display for GroupDefinitionToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            GroupDefinitionToken::Keyword => "group",
            GroupDefinitionToken::Identifier => "identifier",
            GroupDefinitionToken::SyntaxError => "error",
            GroupDefinitionToken::Pipe => "|",
            GroupDefinitionToken::Quantifier(_) => "quantifier",
            GroupDefinitionToken::NewLine => "newline",
            GroupDefinitionToken::Whitespace => "whitespace",
            GroupDefinitionToken::Semicolon => ";",
        };

        write!(f, "{str}")
    }
}

impl StatementTokens for GroupDefinitionToken {
    fn keyword() -> &'static str {
        "group"
    }

    fn keyword_token() -> Self {
        Self::Keyword
    }
}

impl NamedStatement for GroupDefinitionToken {
    fn identifier() -> Self {
        Self::Identifier
    }
}

pub(crate) fn group_callback(lex: &mut Lexer<SchemaStatement>) -> Vec<Token<GroupDefinitionToken>> {
    let mut tokens = Vec::new();
    Token::push_with_advance(&mut tokens, GroupDefinitionToken::Keyword, lex);

    let mut inner = lex.clone().morph::<GroupDefinitionToken>();
    while let Some(token) = inner.next() {
        match unwrap_or_continue!(token, &mut tokens, GroupDefinitionToken::SyntaxError, &mut inner) {
            GroupDefinitionToken::NewLine => inner.extras.new_line(),
            GroupDefinitionToken::Semicolon => push_and_break!(&mut tokens, GroupDefinitionToken::Semicolon, &mut inner),
            GroupDefinitionToken::Whitespace => inner.extras.advance(inner.span().len() as u32),
            kind => Token::push_with_advance(&mut tokens, kind, &mut inner),
        }
    }

    *lex = inner.morph();
    tokens
}

#[derive(Logos, Debug, PartialEq, Eq, Clone)]
#[logos(extras = Position)]
#[logos(error(Error, Error::from_lexer))]
pub enum QuantifierToken {
    #[token("?")]
    ZeroOrOne,

    #[token("*")]
    ZeroOrMore,

    #[token("+")]
    OneOrMore,

    #[token("[")]
    LeftSquareBracket,

    #[token("]")]
    RightSquareBracket,

    #[token("..")]
    Range,

    #[regex(r"\d+")]
    Number,

    #[token("\n")]
    NewLine,

    #[regex(r"[ \t\r]+")]
    Whitespace,

    SyntaxError,
}

impl Display for QuantifierToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            QuantifierToken::ZeroOrOne => "?",
            QuantifierToken::ZeroOrMore => "*",
            QuantifierToken::OneOrMore => "+",
            QuantifierToken::LeftSquareBracket => "[",
            QuantifierToken::RightSquareBracket => "]",
            QuantifierToken::Range => "..",
            QuantifierToken::Number => "number",
            QuantifierToken::NewLine => "newline",
            QuantifierToken::Whitespace => "whitespace",
            QuantifierToken::SyntaxError => "syntax error",
        };

        write!(f, "{str}")
    }
}

fn quantifier_callback(lex: &mut Lexer<GroupDefinitionToken>) -> Vec<Token<QuantifierToken>> {
    let mut tokens = Vec::new();
    match lex.slice() {
        "?" => {
            Token::push_with_advance(&mut tokens, QuantifierToken::ZeroOrOne, lex);
            return tokens;
        }
        "*" => {
            Token::push_with_advance(&mut tokens, QuantifierToken::ZeroOrMore, lex);
            return tokens;
        }
        "+" => {
            Token::push_with_advance(&mut tokens, QuantifierToken::OneOrMore, lex);
            return tokens;
        }
        "[" => Token::push_with_advance(&mut tokens, QuantifierToken::LeftSquareBracket, lex),
        _ => panic!(),
    }

    let mut inner = lex.clone().morph::<QuantifierToken>();
    while let Some(token) = inner.next() {
        match unwrap_or_continue!(token, &mut tokens, QuantifierToken::SyntaxError, &mut inner) {
            QuantifierToken::NewLine => inner.extras.new_line(),
            QuantifierToken::RightSquareBracket => push_and_break!(&mut tokens, QuantifierToken::RightSquareBracket, &mut inner),
            QuantifierToken::Whitespace => inner.extras.advance(inner.span().len() as u32),
            kind => Token::push_with_advance(&mut tokens, kind, &mut inner),
        }
    }

    *lex = inner.morph();
    tokens
}
