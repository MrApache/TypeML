#![allow(clippy::cast_possible_truncation)]

mod tokens;

use crate::tokens::get_tokens;
use rmlx::{AnalysisWorkspace, RmlxParser, SchemaAst};
use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::Path;
use std::sync::{Arc, RwLock};
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::{
    CompletionItem, CompletionOptions, CompletionParams, CompletionResponse, DeclarationCapability, DeclarationOptions,
    DeclarationRegistrationOptions, DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
    DocumentFilter, InitializeParams, InitializeResult, InitializedParams, InsertTextFormat, MessageType,
    SemanticTokenType, SemanticTokens, SemanticTokensFullOptions, SemanticTokensLegend, SemanticTokensOptions,
    SemanticTokensParams, SemanticTokensResult, SemanticTokensServerCapabilities, ServerCapabilities,
    StaticRegistrationOptions, TextDocumentRegistrationOptions, TextDocumentSyncCapability, TextDocumentSyncKind, Url,
    WorkDoneProgressOptions,
};
use tower_lsp::{Client, LanguageServer, LspService, Server};

struct Backend {
    client: Client,

    schemas: RwLock<HashMap<Url, AnalysisWorkspace>>, // RMLX files
    workspaces: RwLock<HashMap<Url, Workspace>>,      // RML  files
}

struct Workspace {
    _references: Vec<Arc<SchemaAst>>,
    content: String,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                declaration_provider: Some(DeclarationCapability::RegistrationOptions(
                    DeclarationRegistrationOptions {
                        text_document_registration_options: TextDocumentRegistrationOptions {
                            document_selector: Some(vec![
                                DocumentFilter {
                                    language: Some("rust-markup-language".to_string()),
                                    scheme: None,
                                    pattern: Some("*.{rml}".to_string()),
                                },
                                DocumentFilter {
                                    language: Some("rust-markup-language-expressions".to_string()),
                                    scheme: None,
                                    pattern: Some("*.{rmlx}".to_string()),
                                },
                            ]),
                        },
                        declaration_options: DeclarationOptions {
                            work_done_progress_options: WorkDoneProgressOptions {
                                work_done_progress: Some(false),
                            },
                        },
                        static_registration_options: StaticRegistrationOptions { id: None },
                    },
                )),
                text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(true),
                    trigger_characters: Some(vec!["#".to_string()]),
                    ..Default::default()
                }),
                semantic_tokens_provider: Some(SemanticTokensServerCapabilities::SemanticTokensOptions(
                    SemanticTokensOptions {
                        legend: SemanticTokensLegend {
                            token_types: vec![
                                SemanticTokenType::KEYWORD,
                                SemanticTokenType::PARAMETER,
                                SemanticTokenType::STRING,
                                SemanticTokenType::TYPE,
                                SemanticTokenType::OPERATOR,
                                SemanticTokenType::NUMBER,
                                SemanticTokenType::COMMENT,
                                SemanticTokenType::MACRO,
                                SemanticTokenType::FUNCTION,
                            ],
                            token_modifiers: vec![],
                        },
                        //full: Some(SemanticTokensFullOptions::Delta {
                        //    delta: None,
                        //}),
                        full: Some(SemanticTokensFullOptions::Bool(true)),
                        //range: Some(true),
                        ..Default::default()
                    },
                )),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client.log_message(MessageType::INFO, "server initialized!").await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let extension = Path::new(uri.path()).extension().and_then(|e| e.to_str()).unwrap();

        match extension {
            "rml" => {
                /*
                let stream = RmlTokenStream::new(&params.text_document.text);
                let tokens = stream.to_vec();
                let mut workspaces = self.workspaces.write().unwrap();
                workspaces.insert(
                    uri,
                    Workspace {
                        _references: vec![],
                        content: params.text_document.text,
                        tokens,
                    },
                );
                 */
            }
            "rmlx" => {
                let workspace = AnalysisWorkspace::new(uri.clone()).run();
                let mut schemas = self.schemas.write().unwrap();
                schemas.insert(uri, workspace);
                //self.client.publish_diagnostics(uri.clone(), std::mem::take(&mut model.diagnostics), None).await;
            }
            _ => unreachable!("Unsupported file type '{extension}'"),
        }
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        let text = &params.content_changes[0].text;
        let extension = Path::new(uri.path()).extension().and_then(OsStr::to_str).unwrap();

        match extension {
            "rml" => {
                /*
                let mut write = self.workspaces.write().unwrap();
                let file = write.get_mut(&uri).unwrap();
                file.tokens = RmlTokenStream::new(text).to_vec();
                file.content.clone_from(text);
                 */
            }
            "rmlx" => {
                let workspace = AnalysisWorkspace::new(uri.clone()).run();
                let mut schemas = self.schemas.write().unwrap();
                schemas.insert(uri, workspace);
                //self.client.publish_diagnostics(uri.clone(), std::mem::take(&mut schema.diagnostics), None).await;
            }
            _ => unreachable!(),
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let _ = params;
        //let mut write = self.workspaces.write().unwrap();
        //write.remove(&params.text_document.uri).unwrap();
    }

    async fn semantic_tokens_full(&self, params: SemanticTokensParams) -> Result<Option<SemanticTokensResult>> {
        self.client.log_message(MessageType::INFO, "Send semantic tokens").await;

        let extension = Path::new(params.text_document.uri.path())
            .extension()
            .and_then(OsStr::to_str)
            .unwrap();

        let tokens = match extension {
            "rmlx" => {
                let read = self.schemas.read().unwrap();
                let content = read.get(&params.text_document.uri).unwrap().source();
                dbg!(content);
                let cst = RmlxParser::build_cst(content);
                get_tokens(&cst)
            }
            "rml" => vec![],
            _ => unreachable!(),
        };

        Ok(Some(SemanticTokensResult::Tokens(SemanticTokens {
            result_id: None,
            data: tokens,
        })))
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        if let Some(context) = params.context {
            if let Some(trigger) = context.trigger_character {
                match trigger.as_str() {
                    "#" => Ok(Some(CompletionResponse::Array(vec![
                        CompletionItem {
                            label: "#import".into(),
                            insert_text: Some("#import \"$0\"".to_string()),
                            insert_text_format: Some(InsertTextFormat::SNIPPET),
                            ..Default::default()
                        },
                        CompletionItem {
                            label: "#expressions".into(),
                            insert_text: Some("#expressions \"$0\" as $1".to_string()),
                            insert_text_format: Some(InsertTextFormat::SNIPPET),
                            ..Default::default()
                        },
                    ]))),
                    _ => Ok(None),
                }
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| Backend {
        client,
        schemas: RwLock::default(),
        workspaces: RwLock::default(),
    });
    Server::new(stdin, stdout, socket).serve(service).await;
}

#[cfg(test)]
mod tests {
    use crate::tokens::get_tokens;
    use rmlx::RmlxParser;

    #[test]
    fn semantic_tokens() {
        let content = std::fs::read_to_string("/home/irisu/Storage/Projects/rml/examples/base.rmlx").unwrap();
        let cst = RmlxParser::build_cst(&content);
        let _tokens = get_tokens(&cst);
        println!();
    }
}
