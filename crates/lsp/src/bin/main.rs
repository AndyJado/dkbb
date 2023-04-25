use std::fs;
use std::sync::{Arc, RwLock};

use ide::AnalysisHost;
use ide_db::ir::{Diagnostics, Program};
use ide_db::line_index::LineIndex;
use lsp::helper::{range, user_edit};

use syntax::parse::parse_text;

use serde_json::Value;
use text_edit::Indel;
use tower_lsp::jsonrpc::{Error, Result};
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

#[derive(Debug)]
struct GlobalState {
    client: Client,
    pub(crate) analysis_host: AnalysisHost,
    vfs: Arc<RwLock<String>>, // pub(crate) diagnostics: DiagnosticCollection,
}

impl GlobalState {
    fn file(&self) -> std::sync::RwLockReadGuard<'_, std::string::String> {
        self.vfs.read().unwrap()
    }
    fn db(&self) -> std::sync::MutexGuard<'_, ide_db::RootDatabase> {
        self.analysis_host.db()
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
        let _version = 0;
        let _language_id = "dyna".to_string();

        // parse whole file
        let cst_parse = parse_text(&text);

        // again, async issue
        {
            let mut f = self.vfs.write().unwrap();
            *f = text;
        }

        // salsa input
        let source = Program::new(&*self.db(), cst_parse);
        ide_db::ir::compile(&*self.db(), source);
        let diags = ide_db::ir::compile::accumulated::<Diagnostics>(&*self.db(), source);

        // lsp api
        let line_index = LineIndex::new(&self.file());
        let diags = diags
            .into_iter()
            .map(|e| Diagnostic::new_simple(range(&line_index, e.range()), e.to_string()))
            .collect();
        self.client.publish_diagnostics(uri, diags, None).await;
        self.client
            .log_message(MessageType::INFO, "file opened!")
            .await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let DidChangeTextDocumentParams {
            text_document: VersionedTextDocumentIdentifier { uri, version: _ },
            content_changes,
        } = params;
        let line_index = LineIndex::new(&self.vfs.read().unwrap());
        let edits = user_edit(&line_index, content_changes);

        let dbg_dg = {
            let dbg_file = fs::read_to_string(uri.path()).unwrap_or("can't read uri".to_string());
            vec![Diagnostic::new_simple(Range::default(), dbg_file)]
        };

        let diags: Vec<Diagnostic> = edits
            .into_iter()
            .map(|c| {
                let Indel { insert, delete } = c;
                Diagnostic::new_simple(range(&line_index, delete), insert)
            })
            .collect();
        // self.on_change(params);
        self.client.publish_diagnostics(uri, dbg_dg, None).await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn completion(&self, _params: CompletionParams) -> Result<Option<CompletionResponse>> {
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
        vfs: Arc::new(RwLock::new(String::new())),
    });
    Server::new(stdin, stdout, socket).serve(service).await;
}
