use crate::cst::RmlNode;
use crate::unresolved::build_ident;
use crate::unresolved::expression::Expression;
use crate::unresolved::structure::Struct;
use lexer_core::CstNode;

#[derive(Debug)]
pub enum ImplKind {
    Expr(Expression),
    Struct(Struct),
}

#[derive(Debug)]
pub struct Impl {
    pub identifier: String,
    pub kind: ImplKind,
}

impl Impl {
    pub fn build(node: &CstNode<RmlNode>) -> Impl {
        let child = node.children.first().unwrap();
        match child.kind {
            RmlNode::ExprImpl => build_expr_impl(child),
            RmlNode::StructImpl => build_struct_impl(child),
            _ => unreachable!(),
        }
    }
}

fn build_expr_impl(node: &CstNode<RmlNode>) -> Impl {
    let mut iter = node.children.iter();
    let ident = iter.next().unwrap().text.clone();
    let (definition_ns, definition_ident) = build_ident(iter.next().unwrap());
    let arguments = Expression::build_expression_arguments(iter.next().unwrap());

    let expr = Expression {
        namespace: definition_ns,
        identifier: definition_ident,
        arguments,
    };

    Impl {
        identifier: ident,
        kind: ImplKind::Expr(expr),
    }
}

fn build_struct_impl(node: &CstNode<RmlNode>) -> Impl {
    let mut iter = node.children.iter();
    let identifier = iter.next().unwrap().text.clone();
    let (definition_ns, definition_ident) = build_ident(iter.next().unwrap());
    let fields_node = iter.next().unwrap();
    let fields = Struct::build_struct_fields(fields_node);

    let r#struct = Struct {
        source: fields_node.text.clone(),
        fields,
    };

    Impl {
        identifier,
        kind: ImplKind::Struct(r#struct),
    }
}
