use std::fs;
use std::sync::{Arc, Mutex};

use lsp::helper::{range, user_edit};
use lsp::ir::{compile, Diagnostics, Diff, SourceProgram};
use lsp::line_index::LineIndex;
use lsp::{Db, RootDatabase};

use serde_json::Value;

use text_edit::Indel;
use tower_lsp::jsonrpc::{Error, Result};
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

#[derive(Debug)]
struct GlobalState {
    client: Client,
    pub(crate) analysis_host: Arc<Mutex<RootDatabase>>,
}

impl GlobalState {
    fn new(client: Client) -> Self {
        Self {
            client,
            analysis_host: Arc::new(Mutex::new(RootDatabase::new())),
        }
    }
    fn db(&self) -> std::sync::MutexGuard<'_, lsp::RootDatabase> {
        self.analysis_host.lock().unwrap()
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for GlobalState {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: None,
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Options(
                    TextDocumentSyncOptions {
                        open_close: None,
                        change: Some(TextDocumentSyncKind::INCREMENTAL),
                        will_save: None,
                        will_save_wait_until: None,
                        save: Some(TextDocumentSyncSaveOptions::SaveOptions(SaveOptions {
                            include_text: Some(false),
                        })),
                    },
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

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        let DidSaveTextDocumentParams {
            text_document,
            text: _,
        } = params;
        let uri = text_document.uri;
        let source = self.db().input(uri.path());
        // again, async issue
        // salsa input
        compile(&*self.db(), source, None);
        let diags = compile::accumulated::<Diagnostics>(&*self.db(), source, None);
        self.client.publish_diagnostics(uri, diags, None).await;
    }

    // XXX
    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let _version = 0;
        let _language_id = "dyna".to_string();

        let source = self.db().input(uri.path());
        // dead locked
        // {
        //     self.analysis_host.lock().unwrap().cst = Some(source.node(&*self.db()).clone());
        // }
        // again, async issue
        // salsa input
        compile(&*self.db(), source, None);
        let diags = compile::accumulated::<Diagnostics>(&*self.db(), source, None);
        self.client.publish_diagnostics(uri, diags, None).await;
        self.client
            .log_message(MessageType::INFO, "file opened!")
            .await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let DidChangeTextDocumentParams {
            text_document: VersionedTextDocumentIdentifier { uri, version: _ },
            content_changes: _,
        } = params;
        // let edits = user_edit(&line_index, content_changes);

        let dbg_dg = {
            let dbg_file = fs::read_to_string(uri.path()).unwrap_or("can't read uri".to_string());
            let green = &self.db().cst;
            let duh = format!("{:?}", green);
            vec![Diagnostic::new_simple(Range::default(), duh)]
        };

        // let diags: Vec<Diagnostic> = edits
        //     .into_iter()
        //     .map(|c| {
        //         let Indel { insert, delete } = c;
        //         Diagnostic::new_simple(range(&line_index, delete), insert)
        //     })
        //     .collect();
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

    let (service, socket) = LspService::new(|client| GlobalState::new(client));
    Server::new(stdin, stdout, socket).serve(service).await;
}
