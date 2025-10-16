use crate::cst::RmlNode;
use lexer_core::CstNode;

mod attribute;
mod directive;
mod element;
mod expression;
mod implements;
mod structure;

pub use attribute::*;
pub use directive::*;
pub use element::*;
pub use expression::*;
pub use implements::*;
pub use structure::*;

#[derive(Debug)]
pub struct LayoutAst {
    pub directives: Vec<Directive>,
    pub root: Option<Element>,
    pub impls: Vec<Impl>,
}

impl LayoutAst {
    #[must_use]
    pub fn build(cst: &CstNode<RmlNode>) -> LayoutAst {
        let mut impls = Vec::new();
        let mut directives = Vec::new();
        let mut root = None;

        for child in &cst.children {
            match child.kind {
                RmlNode::Directive => directives.push(Directive::build(child)),
                RmlNode::Element => root = Some(Element::build(child)),
                RmlNode::Impls => impls.push(Impl::build(child)),
                RmlNode::Symbol => {}
                _ => unreachable!("{:#?}", child.kind),
            }
        }

        LayoutAst {
            directives,
            root,
            impls,
        }
    }
}

fn build_ident(node: &CstNode<RmlNode>) -> (Option<String>, String) {
    match node.kind {
        RmlNode::Ident => (None, node.text.to_string()),
        RmlNode::NsIdent => {
            if let Some((ns, ident)) = node.text.rsplit_once("::") {
                (Some(ns.to_string()), ident.to_string())
            } else {
                (None, node.text.to_string())
            }
        }
        _ => unreachable!(),
    }
}
