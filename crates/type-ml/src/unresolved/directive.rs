use crate::cst::RmlNode;
use lexer_core::CstNode;

#[derive(Debug)]
pub struct Directive {
    pub name: String,
    pub value: Option<String>,
}

impl Directive {
    pub fn build(node: &CstNode<RmlNode>) -> Self {
        let mut iter = node.children.iter();
        let name = iter.next().unwrap().text.clone();
        let value = iter.next().map(|n| Some(n.text.clone())).unwrap_or_default();
        Directive { name, value }
    }
}
