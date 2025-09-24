mod cst;
mod ast;

pub use ast::*;
pub use cst::*;

use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct RmlxParser;

impl RmlxParser {
    #[must_use]
    pub fn build_cst(content: &str) -> CstNode{
        let mut prev_line = 1;
        let mut prev_col = 1;

        let mut parse_tree = RmlxParser::parse(Rule::file, content).unwrap();
        build_cst(&parse_tree.next().unwrap(), &mut prev_line, &mut prev_col)
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
        let ast = RmlxParser::build_ast(CONTENT);
        dbg!(ast);
    }
}
