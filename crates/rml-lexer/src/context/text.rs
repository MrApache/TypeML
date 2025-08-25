use logos::{Lexer, Logos};

use crate::{DefaultContext, Position, Token};

#[derive(Logos, Debug, PartialEq, Eq, Clone)]
#[logos(extras = Position)]
pub enum Text {
    #[token("\n")]
    Newline,

    #[regex(r"[^\n/#<]+")]
    Other,

    #[token("/")]
    End1,

    #[token("#")]
    End2,

    #[token("<")]
    End3,
}

pub(crate) fn text_context_callback(
    lex: &mut Lexer<DefaultContext>,
) -> Option<Token<Text>> {

    //Skip first character
    lex.extras.current_column += 1;
    let mut chars = 1;
    let mut bytes = 0;

    let delta_start = lex.extras.get_delta_start();
    let delta_line = lex.extras.get_delta_line();
    let start = lex.span().start;

    if lex.slice().eq("\n") {
        lex.extras.new_line();
    }

    let mut inner = lex.clone().morph::<Text>();
    for ch in inner.remainder().chars() {
        match ch {
            '/' | '#' | '<' => break,
            '\n' => {
                chars += 1;
                bytes += '\n'.encode_utf8(&mut [0; 2]).len();
                inner.extras.new_line();
            },
            _ => {
                chars += 1;
                inner.extras.current_column += 1;
                bytes += ch.encode_utf8(&mut [0; 2]).len();
            },
        }
    }

    inner.bump(bytes);

    *lex = inner.morph();
    Some(Token {
        kind: Text::Other,
        span: start..start + chars,
        delta_line,
        delta_start,
    })
}
