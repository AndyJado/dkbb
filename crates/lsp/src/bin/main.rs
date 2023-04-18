use ropey::Rope;

use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tower_lsp::jsonrpc::{Error, Result};
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

#[derive(Debug)]
struct Backend {
    client: Client,
    ast_map: DashMap<String, Location>,
    symbol_map: DashMap<String, Location>,
    document_map: DashMap<String, Rope>,
}

impl Backend {
    async fn on_change(&self, params: TextDocumentItem) {
        let rope = ropey::Rope::from_str(&params.text);
        self.document_map
            .insert(params.uri.to_string(), rope.clone());
        let diag_msg = format!("{:?}", self.symbol_map);
        let diag = Diagnostic::new_simple(Range::default(), diag_msg);
        let diags = vec![diag];
        self.client
            .publish_diagnostics(params.uri.clone(), diags, None)
            .await;
        // parse file
    }
} 
#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: None,
            capabilities: ServerCapabilities {
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                color_provider: Some(ColorProviderCapability::Simple(true)),
                document_symbol_provider: Some(OneOf::Right(DocumentSymbolOptions {
                    label: Some("lable".to_string()),
                    work_done_progress_options: WorkDoneProgressOptions {
                        work_done_progress: None,
                    },
                })),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(true),
                    trigger_characters: Some(vec!["*".to_string()]),
                    all_commit_characters: None,
                    work_done_progress_options: WorkDoneProgressOptions {
                        work_done_progress: None,
                    },
                }),

                execute_command_provider: Some(ExecuteCommandOptions {
                    commands: vec!["custom.notification".to_string()],
                    work_done_progress_options: WorkDoneProgressOptions {
                        work_done_progress: Some(true),
                    },
                }),
                ..ServerCapabilities::default()
            },
            ..Default::default()
        })
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let text = params.text_document.text;
        let version = 0;
        let language_id = "dyna".to_string();
        self.on_change(TextDocumentItem {
            uri,
            text,
            version,
            language_id,
        })
        .await;
        self.client
            .log_message(MessageType::INFO, "file opened!")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let item = CompletionItem::new_simple("new".to_string(), "sim".to_string());
        let remains = vec![item];
        let res = CompletionResponse::Array(remains);
        Ok(Some(res))
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        let uri = params.text_document.uri;
        let info = |k: String, v: Location| -> SymbolInformation {
            SymbolInformation {
                name: k,
                kind: SymbolKind::STRUCT,
                tags: None,
                deprecated: None,
                location: v,
                container_name: Some("container".to_string()),
            }
        };
        let ks: Vec<_> = self
            .symbol_map
            .iter()
            .map(|c| info(c.key().clone(), c.value().clone()))
            .collect();
        let res = DocumentSymbolResponse::Flat(ks);
        Ok(Some(res))
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let range = params.text_document_position_params;
        Ok(Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: "正在施工🚧".to_string(),
            }),
            range: Some(Range::new(range.position, range.position)),
        }))
    }

    async fn execute_command(&self, params: ExecuteCommandParams) -> Result<Option<Value>> {
        if params.command == "custom.notification" {
            self.client
                .show_message(MessageType::INFO, "info".to_string())
                .await;
            self.client
                .log_message(
                    MessageType::INFO,
                    format!("Command executed with params: {params:?}"),
                )
                .await;
            Ok(None)
        } else {
            Err(Error::invalid_request())
        }
    }
}

#[tokio::main]
async fn main() {
    #[cfg(feature = "runtime-agnostic")]
    use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};

    tracing_subscriber::fmt().init();

    let (stdin, stdout) = (tokio::io::stdin(), tokio::io::stdout());
    #[cfg(feature = "runtime-agnostic")]
    let (stdin, stdout) = (stdin.compat(), stdout.compat_write());

    let (service, socket) = LspService::new(|client| Backend {
        client,
        ast_map: DashMap::new(),
        symbol_map: DashMap::new(),
        document_map: DashMap::new(),
    });
    Server::new(stdin, stdout, socket).serve(service).await;
}
