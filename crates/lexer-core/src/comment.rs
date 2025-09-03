use logos::{Lexer, Logos};
use crate::{Position, Token};

#[derive(Logos, Debug, PartialEq, Eq, Clone)]
#[logos(extras = Position)]
pub enum CommentToken {
    StartLine,
    EndLine,

    StartBlock,
    EndBlock,

    Text,
}

pub fn comment_callback<'source, T>(lex: &mut Lexer<'source, T>) -> Option<Vec<Token<CommentToken>>>
where
    T: Logos<'source, Extras = Position, Source = str>,
    T: Clone,
{
    let mut tokens = Vec::new();
    let mut inner = lex.clone().morph::<CommentToken>();
    let mut iter = inner.remainder().chars().peekable();

    let mut bytes = 0;
    let mut last_token_pos = inner.span().start;

    let special_char = iter.next()?;
    bytes += special_char.encode_utf8(&mut [0; 2]).len();
    let is_line = match special_char {
        '*' => false,
        '/' => true,
        _ => return None,
    };

    let token_kind = if is_line {
        CommentToken::StartLine
    } else {
        CommentToken::StartBlock
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
                    CommentToken::Text,
                    &mut inner,
                    last_token_pos..last_token_pos + chars,
                ));

                inner.extras.advance(chars as u32);
                last_token_pos += chars;

                tokens.push(Token::new_with_span(
                    CommentToken::EndLine,
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
                            CommentToken::Text,
                            &mut inner,
                            last_token_pos..last_token_pos + chars,
                            delta_line_position,
                            text_delta_start,
                        ));

                        last_token_pos += chars;

                        tokens.push(Token::new_with_span(
                            CommentToken::EndBlock,
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
