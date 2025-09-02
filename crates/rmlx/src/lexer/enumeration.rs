use crate::{
    attribute_callback, AttributeToken, Error, NamedStatement, SchemaStatement, StatementTokens,
    TokenBodyStatement,
};
use lexer_core::{push_and_break, unwrap_or_continue, Position, Token};
use logos::{Lexer, Logos};
use std::fmt::Display;

#[derive(Logos, Debug, PartialEq, Eq, Clone)]
#[logos(extras = Position)]
#[logos(error(Error, Error::from_lexer))]
pub enum EnumDefinitionToken {
    Keyword,

    #[regex("[a-zA-Z_][a-zA-Z0-9_]*")]
    Identifier,

    #[token("{")]
    LeftCurlyBracket,

    #[token("}")]
    RightCurlyBracket,

    #[token("(")]
    LeftParenthesis,

    #[token(")")]
    RightParenthesis,

    #[token(",")]
    Comma,

    #[token("|", rule_callback)]
    Rule(Vec<Token<RuleToken>>),

    #[token("#", attribute_callback)]
    Attribute(Vec<Token<AttributeToken>>),

    #[token("\n")]
    NewLine,

    #[regex(r"[ \t\r]+")]
    Whitespace,

    SyntaxError,
}

impl Display for EnumDefinitionToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            EnumDefinitionToken::Keyword => "enum",
            EnumDefinitionToken::Identifier => "identifier",
            EnumDefinitionToken::LeftCurlyBracket => "{",
            EnumDefinitionToken::RightCurlyBracket => "}",
            EnumDefinitionToken::LeftParenthesis => "(",
            EnumDefinitionToken::RightParenthesis => ")",
            EnumDefinitionToken::Comma => ",",
            EnumDefinitionToken::Rule(_) => "rule",
            EnumDefinitionToken::Attribute(_) => "attribute",
            EnumDefinitionToken::NewLine => "newline",
            EnumDefinitionToken::Whitespace => "whitespace",
            EnumDefinitionToken::SyntaxError => "syntax error",
        };

        write!(f, "{str}")
    }
}

impl StatementTokens for EnumDefinitionToken {
    fn keyword() -> &'static str {
        "enum"
    }

    fn keyword_token() -> Self {
        Self::Keyword
    }
}

impl TokenBodyStatement for EnumDefinitionToken {
    fn left_curly_bracket() -> Self {
        Self::LeftCurlyBracket
    }

    fn right_curly_bracket() -> Self {
        Self::RightCurlyBracket
    }
}

impl NamedStatement for EnumDefinitionToken {
    fn identifier() -> Self {
        Self::Identifier
    }
}

pub(crate) fn enum_callback(lex: &mut Lexer<SchemaStatement>) -> Vec<Token<EnumDefinitionToken>> {
    let mut tokens = Vec::new();
    Token::push_with_advance(&mut tokens, EnumDefinitionToken::Keyword, lex);

    let mut bracket_depth = 0;
    let mut inner = lex.clone().morph::<EnumDefinitionToken>();
    while let Some(token) = inner.next() {
        match unwrap_or_continue!(
            token,
            &mut tokens,
            EnumDefinitionToken::SyntaxError,
            &mut inner
        ) {
            EnumDefinitionToken::NewLine => inner.extras.new_line(),
            EnumDefinitionToken::Whitespace => inner.extras.advance(inner.span().len() as u32),
            kind => {
                if let EnumDefinitionToken::LeftCurlyBracket = &kind {
                    bracket_depth += 1;
                } else if let EnumDefinitionToken::RightCurlyBracket = &kind {
                    if bracket_depth == 0 {
                        Token::push_with_advance(
                            &mut tokens,
                            EnumDefinitionToken::SyntaxError,
                            &mut inner,
                        );
                        return tokens;
                    }
                    bracket_depth -= 1;
                    if bracket_depth == 0 {
                        push_and_break!(&mut tokens, kind, &mut inner);
                    }
                }
                Token::push_with_advance(&mut tokens, kind, &mut inner);
            }
        }
    }

    *lex = inner.morph();
    tokens
}

#[derive(Logos, Debug, PartialEq, Eq, Clone)]
#[logos(extras = Position)]
#[logos(error(Error, Error::from_lexer))]
pub enum RuleToken {
    Pipe,

    //#[regex("[a-zA-Z_][a-zA-Z0-9_]*")]
    //Identifier,

    #[regex(r#""([^"\\]|\\.)*""#)]
    String,

    #[token("\n")]
    NewLine,

    #[regex(r"[ \t\r]+")]
    Whitespace,

    SyntaxError,
}

fn rule_callback(lex: &mut Lexer<EnumDefinitionToken>) -> Vec<Token<RuleToken>> {
    let mut tokens = Vec::new();
    Token::push_with_advance(&mut tokens, RuleToken::Pipe, lex);

    let mut inner = lex.clone().morph::<RuleToken>();
    while let Some(token) = inner.next() {
        match unwrap_or_continue!(token, &mut tokens, RuleToken::SyntaxError, &mut inner) {
            RuleToken::NewLine => inner.extras.new_line(),
            RuleToken::Whitespace => inner.extras.advance(inner.span().len() as u32),
            RuleToken::String => {
                Token::push_with_advance(&mut tokens, RuleToken::String, &mut inner);
                break;
            }
            kind => Token::push_with_advance(&mut tokens, kind, &mut inner),
        }
    }

    *lex = inner.morph();
    tokens
}
