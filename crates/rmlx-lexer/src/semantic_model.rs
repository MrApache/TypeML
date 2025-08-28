pub struct SchemaModel {
    groups: Vec<Group>,
    tokens: Vec<SemanticToken>,
}

impl SchemaModel {
    pub fn new(content: &str) -> Result<Self, Error> {
        let mut stream = RmlxTokenStream::new(content);
        let mut sm_tokens = vec![];
        let mut attributes = vec![];

        while let Some(token) = stream.next_token() {
            let token = token?;
            match token {
                crate::SchemaTokens::Attribute(tokens) => {
                    match parse_attributes(tokens.iter(), content, &mut sm_tokens) {
                        Ok(attr) => attributes.push(attr),
                        Err(err) => panic!("Error: {err}")
                    }
                }
                crate::SchemaTokens::Group(tokens) => {}
                crate::SchemaTokens::Element(tokens) => todo!(),
                crate::SchemaTokens::Expression(tokens) => todo!(),
                crate::SchemaTokens::Enum(tokens) => todo!(),
                crate::SchemaTokens::Struct(tokens) => todo!(),
                crate::SchemaTokens::Use(tokens) => todo!(),
                crate::SchemaTokens::NewLine => {}, //skip
                crate::SchemaTokens::Whitespace => {} //skip
            }

        }
        Err(Error::MissingOpeningBrace)
    }
}

use std::path::{Path, PathBuf};
use tower_lsp::lsp_types::SemanticToken;
use url::Url;

use crate::{semantic::parse_attributes, Error, RmlxTokenStream};
use crate::semantic::Group;

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

    Url::from_file_path(&normalized)
        .map_err(|_| format!("Invalid path: {}", normalized.display()))
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
    use crate::semantic_model::to_url;

    #[test]
    fn url() {
        let url = to_url("/home/irisu/file.ext", "../base.ext").unwrap();
        println!("Url: {url}");
    }
}
