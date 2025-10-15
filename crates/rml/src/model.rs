use crate::analyzer::RmlAnalyzer;
use crate::ast::{AttributeValue, Element, Impl, LayoutAst};
use lexer_core::to_url;
use rmlx::{AnalysisWorkspace, SchemaModel};
use std::sync::{Arc, RwLock};
use url::Url;

pub struct LayoutModel;

impl LayoutModel {
    pub fn validate(ast: LayoutAst, path: &str) -> Result<Element, rmlx::Error> {
        let configs = ast
            .directives
            .iter()
            .filter(|d| d.name == "use")
            .map(|d| {
                let value = d.value.as_ref().unwrap();
                to_url(path, value).unwrap()
            })
            .collect::<Vec<_>>();

        let model = load_config_model(configs)?;
        let mut analyzer = RmlAnalyzer::new(model.clone());
        let root = ast.root.unwrap();
        validate_element(&ast.impls, &root, &mut analyzer)?;
        Ok(root)
    }
}

fn load_config_model(configs: Vec<Url>) -> Result<Arc<RwLock<SchemaModel>>, rmlx::Error> {
    assert!(!configs.is_empty(), "Config not found");
    let mut iter = configs.into_iter();
    let workspace = AnalysisWorkspace::new(iter.next().unwrap()).run()?;
    Ok(workspace.model())
}

fn validate_element(impls: &[Impl], element: &Element, analyzer: &mut RmlAnalyzer) -> Result<(), rmlx::Error> {
    let namespace = element.namespace.as_deref();
    let identifier = &element.identifier;
    if analyzer.is_allowed_element(namespace, identifier)? {
        analyzer.enter_element(namespace, identifier)?;
        element.attributes.iter().try_for_each(|attr| {
            match &attr.value {
                AttributeValue::Expression(expr) => {
                    let expr = expr.as_expr(impls);
                    analyzer.is_valid_expression(element.namespace.as_deref(), &element.identifier, expr)
                }
                AttributeValue::Struct(kind) => {
                    let stc = kind.as_struct(impls);
                    analyzer.is_valid_attribute(&attr.identifier, stc.source.as_str())
                }
                other => analyzer.is_valid_attribute(&attr.identifier, other.as_str()),
            }?;
            Ok::<(), rmlx::Error>(())
        })?;
        element
            .children
            .iter()
            .try_for_each(|child| validate_element(impls, child, analyzer))?;
        analyzer.exit_element(namespace, identifier)?;
        Ok(())
    } else {
        panic!("Incorrect element: {}", element.identifier);
    }
}
