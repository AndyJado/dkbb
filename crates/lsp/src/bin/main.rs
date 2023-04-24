use ide::AnalysisHost;
use ide_db::ir::{Diagnostics, SourceProgram};
use syntax::ast::AstNode;
use syntax::dyna_nodes::KeyWord;
use syntax::parse::parse_text;

use serde_json::Value;
use tower_lsp::jsonrpc::{Error, Result};
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

#[derive(Debug)]
struct GlobalState {
    client: Client,
    pub(crate) analysis_host: AnalysisHost,
    // pub(crate) diagnostics: DiagnosticCollection,
}

impl GlobalState {
    async fn on_change(&self, params: TextDocumentItem) {
        let rope = ropey::Rope::from_str(&params.text);
        let diag_msg = format!("{:?}", "duh");
        let diag = Diagnostic::new_simple(Range::default(), diag_msg);
        let diags = vec![diag];
        self.client
            .publish_diagnostics(params.uri.clone(), diags, None)
            .await;
        // parse file
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for GlobalState {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: None,
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::INCREMENTAL,
                )),
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

    // XXX
    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let text = params.text_document.text;
        let version = 0;
        let language_id = "dyna".to_string();
        // first time open should read whole str
        let cst_parse = parse_text(&text);
        self.analysis_host
            .db_with(&|c| ide_db::ir::compile(c, SourceProgram::new(c, text.to_string())));
        // self.analysis_host.diags(source)
        self.client
            .log_message(MessageType::INFO, "file opened!")
            .await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let DidChangeTextDocumentParams {
            text_document: VersionedTextDocumentIdentifier { uri, version },
            content_changes,
        } = params;
        for event in content_changes {
            let TextDocumentContentChangeEvent {
                range: Some(range),
                range_length: _,
                text,
                // assume all change have range
            } = event else {return};
            // i.range
            todo!()
        }
        // self.on_change(params);
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

    let (service, socket) = LspService::new(|client| GlobalState {
        client,
        analysis_host: AnalysisHost::new(),
    });
    Server::new(stdin, stdout, socket).serve(service).await;
}
