use crate::cst::RmlNode;
use crate::unresolved::attribute::Attribute;
use crate::unresolved::build_ident;
use lexer_core::CstNode;

#[derive(Debug)]
pub struct Element {
    pub namespace: Option<String>,
    pub identifier: String,
    pub attributes: Vec<Attribute>,
    pub children: Vec<Element>,
}

impl Element {
    fn build_element_from_tag(node: &CstNode<RmlNode>) -> Element {
        let (open_ns, open_ident) = build_ident(node.children.first().unwrap());
        let (close_ns, close_ident) = build_ident(node.children.last().unwrap());
        let mut alias = String::new();
        let mut children = vec![];
        let mut attributes = vec![];

        node.children[1..node.children.len() - 1]
            .iter()
            .for_each(|c| match c.kind {
                RmlNode::Alias => alias = build_alias(c),
                RmlNode::Element => children.push(Element::build(c)),
                RmlNode::Attribute => attributes.push(Attribute::build(c)),
                kind => unreachable!("{kind:#?}"),
            });

        //assert_eq!(open_ns, close_ns);
        //assert_eq!(open_ident, close_ident);

        Element {
            namespace: open_ns,
            identifier: open_ident,
            attributes,
            children,
        }
    }

    fn build_element_from_empty_tag(node: &CstNode<RmlNode>) -> Element {
        let (namespace, identifier) = build_ident(node.children.first().unwrap());
        let mut attributes = Vec::new();

        for child in node.children.iter().skip(1) {
            match child.kind {
                RmlNode::Attribute => attributes.push(Attribute::build(child)),
                _ => unreachable!(),
            }
        }

        Element {
            namespace,
            identifier,
            attributes,
            children: vec![],
        }
    }

    pub fn build(node: &CstNode<RmlNode>) -> Self {
        let child = node.children.first().unwrap();
        match child.kind {
            RmlNode::Tag => Self::build_element_from_tag(child),
            RmlNode::EmptyTag => Self::build_element_from_empty_tag(child),
            _ => unreachable!(),
        }
    }
}

fn build_alias(node: &CstNode<RmlNode>) -> String {
    node.children.first().unwrap().text.clone()
}
