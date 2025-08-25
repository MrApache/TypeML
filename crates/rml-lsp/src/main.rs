mod parser;
mod schema;

use rml_lexer::context::{AttributeContext, TagContext};
use rml_lexer::{DefaultContext, RmlTokenStream, TokenType};
use std::collections::HashMap;
use std::iter;
use std::sync::RwLock;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

#[derive(Debug)]
struct Backend {
    client: Client,
    files: RwLock<HashMap<Url, FileData>>,
}

#[derive(Debug)]
struct FileData {
    content: String,
    tokens: Vec<DefaultContext>,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        let multiline = params
            .capabilities
            .text_document
            .as_ref()
            .and_then(|td| td.semantic_tokens.as_ref())
            .and_then(|st| st.multiline_token_support)
            .unwrap_or(false);

        self.client.log_message(MessageType::INFO, format!("multiline support: {multiline}")).await;

        Ok(InitializeResult {
            capabilities: ServerCapabilities {
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
        self.client
            .log_message(MessageType::INFO, format!("Opened: {uri}"))
            .await;
        let stream = RmlTokenStream::new(&params.text_document.text);
        let tokens = stream.to_vec();

        //let mut parser = RmlParser::new(&params.text_document.text, uri);
        //let directives = parser.read_directives().unwrap();

        let mut files = self.files.write().unwrap();
        files.insert(
            uri,
            FileData {
                content: params.text_document.text,
                tokens,
            },
        );
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let mut write = self.files.write().unwrap();
        write.remove(&params.text_document.uri).unwrap();
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let mut write = self.files.write().unwrap();
        let file = write.get_mut(&params.text_document.uri).unwrap();
        let text = params.content_changes.last().unwrap().text.clone(); //TODO fix
        file.tokens = RmlTokenStream::new(&text).to_vec();
        file.content = text;
    }

    async fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> tower_lsp::jsonrpc::Result<Option<SemanticTokensResult>> {
        self.client
            .log_message(MessageType::INFO, "Send semantic tokens")
            .await;
        let read = self.files.read().unwrap();
        let file = read.get(&params.text_document.uri).unwrap();
        let tokens: Vec<SemanticToken> = file
            .tokens
            .iter()
            .flat_map(|token| match token {
                DefaultContext::Directive(inner_tokens) => inner_tokens
                    .iter()
                    .map(|t| SemanticToken {
                        delta_line: t.delta_line(),
                        delta_start: t.delta_start(),
                        length: t.length(),
                        token_type: t.kind().get_token_type(),
                        token_modifiers_bitset: 0,
                    })
                    .collect(),
                DefaultContext::Text(token) => {
                    vec![SemanticToken {
                        delta_line: token.delta_line(),
                        delta_start: token.delta_start(),
                        length: token.length(),
                        token_type: u32::MAX,
                        token_modifiers_bitset: 0,
                    }]
                },
                DefaultContext::Tag(inner_tokens) => {
                    inner_tokens.iter().flat_map(|t| {
                        if let TagContext::Attribute(inner_tokens) = t.kind() {
                            inner_tokens.iter().flat_map(|t| {
                                if let AttributeContext::Struct(inner_tokens) = t.kind() {
                                    inner_tokens.iter().map(|t| {
                                        SemanticToken {
                                            delta_line: t.delta_line(),
                                            delta_start: t.delta_start(),
                                            length: t.length(),
                                            token_type: t.kind().get_token_type(),
                                            token_modifiers_bitset: 0,
                                        }
                                    }).collect()
                                }
                                else {
                                    vec![SemanticToken {
                                        delta_line: t.delta_line(),
                                        delta_start: t.delta_start(),
                                        length: t.length(),
                                        token_type: t.kind().get_token_type(),
                                        token_modifiers_bitset: 0,
                                    }]
                                }
                            }).collect()
                        }
                        else {
                            vec![SemanticToken {
                                delta_line: t.delta_line(),
                                delta_start: t.delta_start(),
                                length: t.length(),
                                token_type: t.kind().get_token_type(),
                                token_modifiers_bitset: 0,
                            }]
                        }
                    }).collect()
                },
                DefaultContext::Comment(inner_tokens) => {
                    inner_tokens.iter().map(|t| {
                        SemanticToken {
                            delta_line: t.delta_line(),
                            delta_start: t.delta_start(),
                            length: t.length(),
                            token_type: t.kind().get_token_type(),
                            token_modifiers_bitset: 0,
                        }
                    }).collect()
                }
            })
            .collect();

        Ok(Some(SemanticTokensResult::Tokens(SemanticTokens {
            result_id: None,
            data: tokens,
        })))
    }
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| Backend {
        client,
        files: Default::default(),
    });
    Server::new(stdin, stdout, socket).serve(service).await;
}
