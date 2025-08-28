use crate::semantic::{Element, Enum, Expression, Group, ParserContext, Struct};
use crate::{semantic::parse_attributes, Error, RmlxTokenStream};
use std::path::{Path, PathBuf};
use tower_lsp::lsp_types::SemanticToken;
use url::Url;

#[derive(Default)]
pub struct SchemaModel {
    includes: Vec<Url>,
    enums: Vec<Enum>,
    groups: Vec<Group>,
    structs: Vec<Struct>,
    elements: Vec<Element>,
    expressions: Vec<Expression>,

    tokens: Vec<SemanticToken>,
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
                    match parse_attributes(tokens.iter(), content, &mut schema.tokens) {
                        Ok(attrs) => attributes.extend(attrs),
                        Err(err) => panic!("Error: {err}"),
                    }
                }
                crate::SchemaStatement::Group(tokens) => {
                    let mut group =
                        ParserContext::new(&mut schema.tokens, tokens.iter().peekable(), content)
                            .parse()
                            .unwrap();
                    group.resolve_attributes(&mut attributes);
                    //TODO error
                    attributes.clear();
                    schema.groups.push(group);
                }
                crate::SchemaStatement::Expression(tokens) => {
                    schema.expressions.push(
                        ParserContext::new(&mut schema.tokens, tokens.iter().peekable(), content)
                            .parse()
                            .unwrap(),
                    );
                }
                crate::SchemaStatement::Enum(tokens) => {
                    let mut enumeration = ParserContext::new(&mut schema.tokens, tokens.iter().peekable(), content)
                        .parse()
                        .unwrap();
                    enumeration.resolve_attributes(&mut attributes);
                    //TODO error
                    attributes.clear();
                    schema.enums.push(enumeration);
                }
                crate::SchemaStatement::Struct(tokens) => {
                    let mut structure =
                        ParserContext::new(&mut schema.tokens, tokens.iter().peekable(), content)
                            .parse()
                            .unwrap();
                    structure.resolve_attributes(&mut attributes);
                    //TODO error
                    attributes.clear();
                    schema.structs.push(structure);
                }
                crate::SchemaStatement::Use(tokens) => {
                    let using = ParserContext::new(&mut schema.tokens, tokens.iter().peekable(), content)
                        .parse().unwrap();

                    schema.includes.push(to_url(file, &using.path).unwrap());
                    //TODO check file
                }
                crate::SchemaStatement::Element(tokens) => {
                    let mut element =
                        ParserContext::new(&mut schema.tokens, tokens.iter().peekable(), content)
                            .parse()
                            .unwrap();
                    element.resolve_attributes(&mut attributes);
                    //TODO error
                    attributes.clear();
                    schema.elements.push(element);
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
        let path = concat!(env!("CARGO_WORKSPACE_DIR"), "/examples/schema.rmlx");
        let content = std::fs::read_to_string(path).expect("Failed to read file");
        let _model = SchemaModel::new(path, &content);
        println!();
    }
}
