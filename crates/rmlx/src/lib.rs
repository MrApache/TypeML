#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(unused)]
//#![deny(clippy::unwrap_used)]

mod ast;
mod cst;
mod semantic;

pub use ast::{SchemaAst, build_schema_ast};
pub use cst::RmlxNode;
use lexer_core::CstNode;
pub use pest::*;
use pest_derive::Parser;
pub use semantic::*;
use thiserror::Error;

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct RmlxParser;

impl RmlxParser {
    pub fn build_ast(content: &str) -> Result<SchemaAst, Error> {
        let cst = CstNode::new::<RmlxParser>(content, Rule::file).map_err(Error::PestError)?;
        Ok(build_schema_ast(&cst))
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Root group not found")]
    RootGroupNotFound,

    #[error("Namespace {0} does not exist")]
    NamespaceNotFound(String),

    #[error("{0}")]
    IncorrectPattern(#[from] regex::Error),

    #[error("{0}")]
    ParseBool(#[from] std::str::ParseBoolError),

    #[error("{0}")]
    LoadError(#[from] LoadError),

    #[error("{0}")]
    UrlError(String),

    #[error("{0}")]
    PestError(String),

    #[error("Element {0} not found")]
    ElementNotFound(String),
}
