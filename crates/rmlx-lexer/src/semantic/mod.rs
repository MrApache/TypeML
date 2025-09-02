mod loader;
use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range};

use crate::{semantic::loader::load_rmlx, SchemaAst};

pub struct SchemaModel {
    
}

impl SchemaModel {
    pub async fn new(ast: &mut SchemaAst) -> Self {
        for url in &ast.includes {
            let rmlx_file = load_rmlx(url).await;
            if let Err(error) = rmlx_file {
                ast.diagnostics.push(Diagnostic {
                    range: Range {
                        start: Position {
                            line: 0,
                            character: 0,
                        },
                        end: Position {
                            line: 0,
                            character: 1,
                        },
                    },
                    severity: Some(DiagnosticSeverity::ERROR),
                    message: error.to_string(),
                    ..Default::default()
                });
            }
        }

        SchemaModel {}
    }
}

#[cfg(test)]
mod tests {
    use crate::{SchemaAst, SchemaModel};

    #[tokio::test]
    async fn test() {
        let path = concat!(env!("CARGO_WORKSPACE_DIR"), "examples/schema.rmlx");
        let content = std::fs::read_to_string(path).expect("Failed to read file");
        let mut ast = SchemaAst::new(&content);
        SchemaModel::new(&mut ast).await;
        println!();
    }
}
