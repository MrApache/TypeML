use url::Url;
use std::path::{Path, PathBuf};
use tower_lsp::lsp_types::{Diagnostic, SemanticToken};
use crate::semantic::{Element, Enum, Expression, Group, ParserContext, Struct};
use crate::{Error, RmlxTokenStream};

#[derive(Default, Debug)]
pub struct SchemaModel {
    includes: Vec<Url>,
    enums: Vec<Enum>,
    groups: Vec<Group>,
    structs: Vec<Struct>,
    elements: Vec<Element>,
    expressions: Vec<Expression>,

    pub tokens: Vec<SemanticToken>,
    pub diagnostics: Vec<Diagnostic>,
}

impl SchemaModel {
    pub fn new(file: &str, content: &str) -> Result<Self, Error> {
        let mut schema = SchemaModel::default();
        let mut stream = RmlxTokenStream::new(content);
        let mut attributes = vec![];

        while let Some(token) = stream.next_token() {
            let token = token?;
            match token {
                crate::SchemaStatement::Attribute(tokens) => {
                    let attrs = ParserContext::new(
                        tokens.iter().peekable(),
                        &mut schema.diagnostics,
                        &mut schema.tokens,
                        content,
                    )
                    .parse();

                    if let Some(attrs) = attrs {
                        attributes = attrs;
                    }
                }
                crate::SchemaStatement::Group(tokens) => {
                    let group = ParserContext::new(
                        tokens.iter().peekable(),
                        &mut schema.diagnostics,
                        &mut schema.tokens,
                        content,
                    )
                    .parse();

                    if let Some(mut group) = group {
                        group.set_attributes(std::mem::take(&mut attributes));
                        schema.groups.push(group);
                    }
                }
                crate::SchemaStatement::Expression(tokens) => {
                    let expression = ParserContext::new(
                        tokens.iter().peekable(),
                        &mut schema.diagnostics,
                        &mut schema.tokens,
                        content,
                    )
                    .parse();

                    if let Some(expression) = expression {
                        schema.expressions.push(expression);
                    }
                }
                crate::SchemaStatement::Enum(tokens) => {
                    let enumeration = ParserContext::new(
                        tokens.iter().peekable(),
                        &mut schema.diagnostics,
                        &mut schema.tokens,
                        content,
                    )
                    .parse();

                    if let Some(mut enumeration) = enumeration {
                        enumeration.set_attributes(std::mem::take(&mut attributes));
                        schema.enums.push(enumeration);
                    }
                }
                crate::SchemaStatement::Struct(tokens) => {
                    let structure = ParserContext::new(
                        tokens.iter().peekable(),
                        &mut schema.diagnostics,
                        &mut schema.tokens,
                        content,
                    )
                    .parse();

                    if let Some(mut structure) = structure {
                        structure.set_attributes(std::mem::take(&mut attributes));
                        schema.structs.push(structure);
                    }
                }
                crate::SchemaStatement::Use(tokens) => {
                    let using = ParserContext::new(
                        tokens.iter().peekable(),
                        &mut schema.diagnostics,
                        &mut schema.tokens,
                        content,
                    )
                    .parse();

                    if let Some(using) = using {
                        //TODO check file
                        schema.includes.push(to_url(file, &using.path).unwrap());
                    }
                }
                crate::SchemaStatement::Element(tokens) => {
                    let element = ParserContext::new(
                        tokens.iter().peekable(),
                        &mut schema.diagnostics,
                        &mut schema.tokens,
                        content,
                    )
                    .parse();

                    if let Some(mut element) = element {
                        element.set_attributes(std::mem::take(&mut attributes));
                        schema.elements.push(element);
                    }
                }
                crate::SchemaStatement::NewLine => {}    //skip
                crate::SchemaStatement::Whitespace => {} //skip
            }
        }

        Ok(schema)
    }
}

/// Преобразует `input` в Url, считая, что он указан относительно файла `base`.
fn to_url(base: impl AsRef<Path>, input: &str) -> Result<Url, String> {
    if let Ok(url) = Url::parse(input) {
        return Ok(url);
    }

    let path = Path::new(input);
    let base_dir = base.as_ref().parent().unwrap_or_else(|| Path::new(""));

    let abs_path: PathBuf = if path.is_absolute() {
        path.to_path_buf()
    } else {
        base_dir.join(path)
    };

    let normalized = normalize_path(&abs_path);

    Url::from_file_path(&normalized).map_err(|_| format!("Invalid path: {}", normalized.display()))
}

/// Убирает `.` и `..` из пути без обращения к файловой системе
fn normalize_path(path: &Path) -> PathBuf {
    let mut result = PathBuf::new();
    for comp in path.components() {
        match comp {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                result.pop();
            }
            other => result.push(other.as_os_str()),
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use crate::semantic_model::SchemaModel;
    #[test]
    fn test() {
        let path = concat!(env!("CARGO_WORKSPACE_DIR"), "examples/base.rmlx");
        let content = std::fs::read_to_string(path).expect("Failed to read file");
        let _model = SchemaModel::new(path, &content);
        println!();
    }
}
