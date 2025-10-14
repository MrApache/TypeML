mod ast;
mod cst;

pub use ast::*;
pub use cst::*;

use lexer_core::CstNode;
use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct RmlParser;

impl RmlParser {
    #[must_use]
    pub fn build_cst(content: &str) -> CstNode<RmlNode> {
        let mut prev_line = 1;
        let mut prev_col = 1;
        let mut result = RmlParser::parse(Rule::file, content);
        if let Ok(mut tree) = result {
            CstNode::<RmlNode>::build_cst(&tree.next().unwrap(), content, &mut prev_line, &mut prev_col)
        } else {
            panic!("Error: {result:#?}");
        }
    }

    #[must_use]
    pub fn build_ast(content: &str) -> LayoutAst {
        let cst = Self::build_cst(content);
        build_layout_ast(&cst)
    }
}

#[cfg(test)]
mod tests {
    use crate::pest::RmlParser;

    const CONTENT: &str = r#"
    <Layout id="asd">
        <Node width=10 height={Expression x=1 x="Hello" y=[A, B, C]}/>
    </Layout>
"#;
    #[test]
    fn test() {
        //let content = std::fs::read_to_string("D:\\Projects\\rml\\examples\\base.rmlx").unwrap();
        //let cst = RmlParser::build_cst(&content);
        let ast = RmlParser::build_ast(CONTENT);
        dbg!(ast);
    }
}
