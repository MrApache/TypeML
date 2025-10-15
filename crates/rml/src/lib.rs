#![allow(clippy::missing_panics_doc)]
#![allow(clippy::missing_errors_doc)]
//#![allow(unused)]

mod analyzer;
mod ast;
mod cst;
mod model;

use crate::ast::{LayoutAst, build_layout_ast};
use crate::cst::RmlNode;
pub use crate::model::LayoutModel;
use lexer_core::CstNode;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct RmlParser;

impl RmlParser {
    pub fn build_ast(content: &str) -> Result<LayoutAst, rmlx::Error> {
        let cst = CstNode::new::<RmlParser>(content, Rule::file).map_err(rmlx::Error::PestError)?;
        Ok(build_layout_ast(&cst))
    }
}
