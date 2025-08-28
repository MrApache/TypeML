use crate::MarkupTokens;
use lexer_utils::*;
use logos::{Lexer, Logos};

#[derive(Logos, Debug, PartialEq, Eq, Clone)]
#[logos(extras = Position)]
pub enum CommentContext {
    StartLine,
    EndLine,

    StartBlock,
    EndBlock,

    Text,
}

impl TokenType for CommentContext {
    fn get_token_type(&self) -> u32 {
        6
    }
}

pub(crate) fn comment_context_callback(
    lex: &mut Lexer<MarkupTokens>,
) -> Option<Vec<Token<CommentContext>>> {
    let mut tokens = Vec::new();
    let mut inner = lex.clone().morph::<CommentContext>();
    let mut iter = inner.remainder().chars().peekable();

    let mut bytes = 0;
    let mut last_token_pos = inner.span().start;

    //Define comment kind
    let special_char = iter.next()?;
    bytes += special_char.encode_utf8(&mut [0; 2]).len();
    let is_line = match special_char {
        '*' => false,
        '/' => true,
        _ => return None, // неожиданный токен
    };

    let token_kind = if is_line {
        CommentContext::StartLine
    } else {
        CommentContext::StartBlock
    };

    tokens.push(Token::new_with_span(
        token_kind,
        &mut inner,
        last_token_pos..last_token_pos + 2,
    ));

    inner.extras.advance(2);
    last_token_pos += 2;

    let mut chars = 0;

    if is_line {
        for ch in iter.by_ref() {
            bytes += ch.encode_utf8(&mut [0; 2]).len();
            if ch == '\n' {
                tokens.push(Token::new_with_span(
                    CommentContext::Text,
                    &mut inner,
                    last_token_pos..last_token_pos + chars,
                ));

                inner.extras.advance(chars as u32);
                last_token_pos += chars;

                tokens.push(Token::new_with_span(
                    CommentContext::EndLine,
                    &mut inner,
                    last_token_pos..last_token_pos + 1,
                ));

                inner.extras.new_line();
                break;
            }
            chars += 1;
        }
    } else {
        let delta_line_position = if iter.peek()? == &'\n' {
            iter.next();

            bytes += '\n'.encode_utf8(&mut [0; 2]).len();
            chars += 1;

            inner.extras.new_line();
            inner.extras.get_delta_line()
        } else {
            0
        };

        let text_delta_start = inner.extras.get_delta_start();
        while let Some(ch) = iter.next() {
            bytes += ch.encode_utf8(&mut [0; 2]).len();
            match ch {
                '\n' => inner.extras.new_line(),
                '*' => {
                    if let Some('/') = iter.peek() {
                        bytes += '/'.encode_utf8(&mut [0; 2]).len();

                        tokens.push(Token::new_custom(
                            CommentContext::Text,
                            &mut inner,
                            last_token_pos..last_token_pos + chars,
                            delta_line_position,
                            text_delta_start,
                        ));

                        last_token_pos += chars;

                        tokens.push(Token::new_with_span(
                            CommentContext::EndBlock,
                            &mut inner,
                            last_token_pos..last_token_pos + 2,
                        ));
                        break;
                    }
                }
                _ => inner.extras.advance(1),
            }

            chars += 1;
        }
    }

    *lex = inner.morph();
    lex.bump(bytes);
    Some(tokens)
}
