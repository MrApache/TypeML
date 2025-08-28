use lexer_utils::STRING_TOKEN;
use crate::{semantic::ParserContext, UseToken};

pub struct Use {
    pub path: String,
}

impl<'s> ParserContext<'s, UseToken> {
    pub fn parse(&mut self) -> Result<Use, String> {
        self.consume_keyword()?;

        let t = self.iter.next().ok_or("Expected <...> path")?;
        if t.kind() != &UseToken::Path {
            return Err(format!("Expected path, found {:?}", t.kind()));
        }

        let path = self.src
            .get(t.span())
            .map(trim_angle_brackets)
            .ok_or("Invalid span for path")?
            .to_string();

        self.tokens.push(t.to_semantic_token(STRING_TOKEN));

        Ok(Use { path })
    }
}

fn trim_angle_brackets(path: &str) -> &str {
    path.strip_prefix('<')
        .and_then(|s| s.strip_suffix('>'))
        .unwrap_or(path)
}
