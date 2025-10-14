mod ast;
mod cst;

pub use ast::*;
pub use cst::*;

use lexer_core::CstNode;
use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct RmlxParser;

impl RmlxParser {
    #[must_use]
    pub fn build_cst(content: &str) -> CstNode<RmlxNode> {
        let mut prev_line = 1;
        let mut prev_col = 1;
        let mut result = RmlxParser::parse(Rule::file, content);
        if let Ok(mut tree) = result {
            CstNode::<RmlxNode>::build_cst(&tree.next().unwrap(), content, &mut prev_line, &mut prev_col)
        } else {
            panic!("Error: {result:#?}");
        }
    }

    #[must_use]
    pub fn build_ast(content: &str) -> SchemaAst {
        let cst = Self::build_cst(content);
        build_schema_ast(&cst)
    }
}

#[cfg(test)]
mod tests {
    use crate::pest::RmlxParser;

    const CONTENT: &str = "
#[Description(Message), Iter(0)]
group Root(1) {
    + unique path::Component(*)
    + Container(?)
}
";
    #[test]
    fn test() {
        let content = std::fs::read_to_string("D:\\Projects\\rml\\examples\\base.rmlx").unwrap();
        let cst = RmlxParser::build_cst(&content);
        let ast = RmlxParser::build_ast(CONTENT);
        dbg!(ast);
    }
}
