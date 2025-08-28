use crate::semantic::{Enum, Expression, Group, ParserContext, Struct};
use crate::{semantic::parse_attributes, Error, RmlxTokenStream};
use std::path::{Path, PathBuf};
use tower_lsp::lsp_types::SemanticToken;
use url::Url;

#[derive(Default)]
pub struct SchemaModel {
    groups: Vec<Group>,
    structs: Vec<Struct>,
    enums: Vec<Enum>,
    expressions: Vec<Expression>,
    tokens: Vec<SemanticToken>,
    includes: Vec<Url>,
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
                        Ok(attrs) => attributes = attrs,
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
                    schema.enums.push(
                        ParserContext::new(&mut schema.tokens, tokens.iter().peekable(), content)
                            .parse()
                            .unwrap(),
                    );
                }
                crate::SchemaStatement::Struct(tokens) => {
                    let mut structure =
                        ParserContext::new(&mut schema.tokens, tokens.iter().peekable(), content)
                            .parse()
                            .unwrap();
                    structure.resolve_attributes(&mut attributes);
                    schema.structs.push(structure);
                }
                crate::SchemaStatement::Use(tokens) => {
                    let using = ParserContext::new(&mut schema.tokens, tokens.iter().peekable(), content)
                        .parse().unwrap();

                    schema.includes.push(to_url(file, &using.path).unwrap());
                    //TODO check file
                }
                crate::SchemaStatement::Element(tokens) => todo!(),
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

    const CONTENT: &str = r#"
#[Min(1), Max(1)]
group Root[Container, Component];

group Container[Component, Template];

group Template[Container];

#[Extend, Min(0), Max(1)]
group Component;

expression Resource {
    groups: [Component],
    required: {
        Target: String,
        Path:   String,
    }
}

expression Component {
    groups: [Component],
    required: {
        Target: String,
        Path:   String,
    },
}

expression Item {
    groups: [Component],
    available_in: [Template],
    required: {
        Path: String,
    },
    additional: {
        Converter: String,
        Fallback:  String,
    }
}
"#;

    #[test]
    fn test() {
        let _model = SchemaModel::new(CONTENT);
        println!();
    }
}
