pub mod context;
mod errors;
use lexer_core::{comment_callback, CommentToken, Position, Token};
pub use logos;

use logos::{Lexer, Logos};

use crate::context::{
    directive_callback, tag_context_callback, text_context_callback, DirectiveToken, TagContext, Text,
};

#[derive(Logos, Debug, PartialEq, Eq, Clone)]
#[logos(extras = Position)]
pub enum MarkupTokens {
    #[token("/", comment_callback)]
    Comment(Vec<Token<CommentToken>>),

    #[token("<", tag_context_callback)]
    Tag(Vec<Token<TagContext>>),

    #[token("#", directive_callback)]
    Directive(Vec<Token<DirectiveToken>>),

    #[regex(r"[^/#<]", text_context_callback)]
    Text(Token<Text>),
}

pub struct RmlTokenStream<'a> {
    inner: Lexer<'a, MarkupTokens>,
}

impl<'a> RmlTokenStream<'a> {
    pub fn new(content: &'a str) -> Self {
        Self {
            inner: MarkupTokens::lexer(content),
        }
    }

    pub fn next_token(&mut self) -> Result<MarkupTokens, ()> {
        if let Some(token_kind) = self.inner.next() {
            token_kind
        } else {
            Err(())
        }
    }

    #[must_use]
    pub fn to_vec(mut self) -> Vec<MarkupTokens> {
        let mut vec = vec![];

        while let Ok(token) = self.next_token() {
            vec.push(token);
        }

        vec
    }
}

#[cfg(test)]
mod tests {
    use crate::MarkupTokens;
    use logos::Logos;

    #[test]
    fn test() {
        const CONTENT: &str = r#"
#import "../base.ron"
#import "../schema.ron"
#expressions "../file.rmlx" as expr

<Layout>
    <Node width="60px" height="100px"/>
    <Container id="Background">
        <Node/>
        <Image path="{expr:LoadIcon}">
        <Container id="Text">
            <Text self="{expr:Text}">
            <Tag attribute="{expr:Test}">
        </Container>
    </Container>
</Layout>
"#;
        let mut lexer = MarkupTokens::lexer(CONTENT);

        while let Some(token) = lexer.next() {
            let slice = lexer.slice().trim();
            if let Ok(token) = &token
                && let MarkupTokens::Text(_token) = token
                && slice.is_empty()
            {
                continue;
            }
            println!("{token:?} => {slice:?}");
        }
    }
}
