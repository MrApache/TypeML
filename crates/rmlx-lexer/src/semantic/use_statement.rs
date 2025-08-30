use lexer_utils::STRING_TOKEN;
use crate::{next_or_none, semantic::ParserContext, UseToken};

pub struct Use {
    pub path: String,
}

impl<'s> ParserContext<'s, UseToken> {
    pub fn parse(&mut self) -> Option<Use> {
        self.consume_keyword();

        let t = next_or_none!(self, "Expected <...> path")?;
        if t.kind() != &UseToken::Path {
            self.create_error_message(format!("Expected path, found {:?}", t.kind()));
            return None;
        }

        let path = self.src
            .get(t.span())
            .map(trim_angle_brackets)
            .unwrap()
            .to_string();

        self.tokens.push(t.to_semantic_token(STRING_TOKEN));

        Some(Use { path })
    }
}

fn trim_angle_brackets(path: &str) -> &str {
    path.strip_prefix('<')
        .and_then(|s| s.strip_suffix('>'))
        .unwrap_or(path)
}
