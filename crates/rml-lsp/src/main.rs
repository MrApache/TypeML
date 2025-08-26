use lexer_utils::{Token, TokenType};
mod parser;
mod schema;

use rml_lexer::context::{AttributeContext, TagContext};
use rml_lexer::{MarkupTokens, RmlTokenStream};
use rmlx_lexer::{RmlxTokenStream, SchemaTokens};
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, RwLock};
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

#[derive(Debug)]
struct Backend {
    client: Client,

    schemas: RwLock<HashMap<Url, SchemaDecalration>>, //RMLX files
    workspaces: RwLock<HashMap<Url, Workspace>>,      //RML  files
}

#[derive(Debug)]
struct Workspace {
    references: Vec<Arc<SchemaDecalration>>,
    content: String,
    tokens: Vec<MarkupTokens>,
}

#[derive(Debug)]
struct SchemaDecalration {
    content: String,
    tokens: Vec<SchemaTokens>,
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
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(true),
                    trigger_characters: Some(vec!["#".to_string()]),
                    ..Default::default()
                }),
                semantic_tokens_provider: Some(
                    SemanticTokensServerCapabilities::SemanticTokensOptions(
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
                            full: Some(SemanticTokensFullOptions::Bool(true)),
                            //range: Some(true),
                            ..Default::default()
                        },
                    ),
                ),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "server initialized!")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
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

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let extension = Path::new(uri.path())
            .extension()
            .and_then(|e| e.to_str())
            .unwrap();

        match extension {
            "rml" => {
                let stream = RmlTokenStream::new(&params.text_document.text);
                let tokens = stream.to_vec();
                let mut workspaces = self.workspaces.write().unwrap();
                workspaces.insert(
                    uri,
                    Workspace {
                        references: vec![],
                        content: params.text_document.text,
                        tokens,
                    },
                );
            }
            "rmlx" => {
                let stream = RmlxTokenStream::new(&params.text_document.text);
                let tokens = stream.to_vec();
                let mut schemas = self.schemas.write().unwrap();
                schemas.insert(
                    uri,
                    SchemaDecalration {
                        content: params.text_document.text,
                        tokens,
                    },
                );
            }
            _ => unreachable!("Unsupported file type '{extension}'"),
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let _ = params;
        //let mut write = self.workspaces.write().unwrap();
        //write.remove(&params.text_document.uri).unwrap();
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        let extension = Path::new(uri.path())
            .extension()
            .and_then(|e| e.to_str())
            .unwrap();

        match extension {
            "rml" => {
                let mut write = self.workspaces.write().unwrap();
                let file = write.get_mut(&uri).unwrap();
                let text = params.content_changes.last().unwrap().text.clone(); //TODO fix
                file.tokens = RmlTokenStream::new(&text).to_vec();
                file.content = text;
            }
            "rmlx" => {
                let mut write = self.schemas.write().unwrap();
                let schema = write.get_mut(&uri).unwrap();
                let text = params.content_changes.last().unwrap().text.clone(); //TODO fix
                schema.tokens = RmlxTokenStream::new(&text).to_vec();
                schema.content = text;
            }
            _ => unreachable!(),
        }
    }

    async fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> tower_lsp::jsonrpc::Result<Option<SemanticTokensResult>> {
        self.client
            .log_message(MessageType::INFO, "Send semantic tokens")
            .await;

        let extension = Path::new(params.text_document.uri.path())
            .extension()
            .and_then(|e| e.to_str())
            .unwrap();

        let tokens = match extension {
            "rml" => self.markup_semantic_tokens(params.text_document.uri),
            "rmlx" => self.schema_semantic_tokens(params.text_document.uri),
            _ => unreachable!(),
        };

        Ok(Some(SemanticTokensResult::Tokens(SemanticTokens {
            result_id: None,
            data: tokens,
        })))
    }
}

impl Backend {
    fn schema_semantic_tokens(&self, uri: Url) -> Vec<SemanticToken> {
        let read = self.schemas.read().unwrap();
        let file = read.get(&uri).unwrap();
        file.tokens
            .iter()
            .flat_map(|token| match token {
                SchemaTokens::Group(tokens) => Self::to_semantic_tokens(tokens),
                SchemaTokens::Struct(tokens) => Self::to_semantic_tokens(tokens),
                SchemaTokens::Element(tokens) => Self::to_semantic_tokens(tokens),
                SchemaTokens::Expression(tokens) => Self::to_semantic_tokens(tokens),
                SchemaTokens::Attribute(tokens) => tokens
                    .iter()
                    .flat_map(|token| {
                        if let rmlx_lexer::AttributeContext::Content(tokens) = token.kind() {
                            Self::to_semantic_tokens(tokens)
                        } else {
                            Self::to_semantic_token(token)
                        }
                    })
                    .collect(),
                SchemaTokens::Enum(tokens) => tokens
                    .iter()
                    .flat_map(|token| {
                        if let rmlx_lexer::EnumContext::Attribute(tokens) = token.kind() {
                            tokens
                                .iter()
                                .flat_map(|token| {
                                    if let rmlx_lexer::AttributeContext::Content(tokens) =
                                        token.kind()
                                    {
                                        Self::to_semantic_tokens(tokens)
                                    } else {
                                        Self::to_semantic_token(token)
                                    }
                                })
                                .collect()
                        } else {
                            Self::to_semantic_token(token)
                        }
                    })
                    .collect(),
                _ => vec![],
            })
            .collect()
    }

    fn markup_semantic_tokens(&self, uri: Url) -> Vec<SemanticToken> {
        let read = self.workspaces.read().unwrap();
        let file = read.get(&uri).unwrap();
        file.tokens
            .iter()
            .flat_map(|token| match token {
                MarkupTokens::Directive(inner_tokens) => Self::to_semantic_tokens(inner_tokens),
                MarkupTokens::Text(token) => Self::to_semantic_token(token),
                MarkupTokens::Tag(inner_tokens) => inner_tokens
                    .iter()
                    .flat_map(|t| {
                        if let TagContext::Attribute(inner_tokens) = t.kind() {
                            inner_tokens
                                .iter()
                                .flat_map(|t| match t.kind() {
                                    AttributeContext::Struct(inner_tokens) => {
                                        Self::to_semantic_tokens(inner_tokens)
                                    }
                                    AttributeContext::Expression(inner_tokens) => {
                                        Self::to_semantic_tokens(inner_tokens)
                                    }
                                    _ => Self::to_semantic_token(t),
                                })
                                .collect()
                        } else {
                            Self::to_semantic_token(t)
                        }
                    })
                    .collect(),
                MarkupTokens::Comment(inner_tokens) => Self::to_semantic_tokens(inner_tokens),
            })
            .collect()
    }

    fn to_semantic_tokens<T: TokenType>(tokens: &[Token<T>]) -> Vec<SemanticToken> {
        tokens
            .iter()
            .map(|t| SemanticToken {
                delta_line: t.delta_line(),
                delta_start: t.delta_start(),
                length: t.length(),
                token_type: t.kind().get_token_type(),
                token_modifiers_bitset: 0,
            })
            .collect()
    }

    fn to_semantic_token<T: TokenType>(token: &Token<T>) -> Vec<SemanticToken> {
        vec![SemanticToken {
            delta_line: token.delta_line(),
            delta_start: token.delta_start(),
            length: token.length(),
            token_type: token.kind().get_token_type(),
            token_modifiers_bitset: 0,
        }]
    }
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| Backend {
        client,
        schemas: Default::default(),
        workspaces: Default::default(),
    });
    Server::new(stdin, stdout, socket).serve(service).await;
}
