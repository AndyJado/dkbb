pub mod helper;
pub mod ir;
pub mod line_index;
use core::fmt;
use std::{
    fs,
    sync::{Arc, Mutex},
};

use ir::{Source, SourceProgram};
use line_index::LineIndex;
use salsa::DebugWithDb;
use syntax::{
    dyna_nodes::SourceFile,
    parse::{parse_text, Parse},
};
use text_edit::TextEdit;
use tower_lsp::lsp_types::{Diagnostic, Range};

#[salsa::jar(db = Db)]
pub struct Jar(
    // input
    crate::ir::Source,
    crate::ir::Diff,
    // struct
    crate::ir::Card,
    crate::ir::SourceProgram,
    crate::ir::Diagnostics,
    // fn
    crate::ir::mat_54,
    crate::ir::parse,
    crate::ir::foo,
    crate::ir::compile,
);

#[derive(Default)]
#[salsa::db(crate::Jar)]
pub struct RootDatabase {
    storage: salsa::Storage<Self>,
    pub cst: Option<(LineIndex, Parse<SourceFile>)>,
    logs: Option<Arc<Mutex<Vec<String>>>>,
}

impl RootDatabase {
    pub fn new() -> RootDatabase {
        RootDatabase::default()
    }
}

impl fmt::Debug for RootDatabase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RootDatabase").finish()
    }
}

impl salsa::Database for RootDatabase {
    fn salsa_event(&self, event: salsa::Event) {
        // Log interesting events, if logging is enabled
        if let Some(logs) = &self.logs {
            // don't log boring events
            if let salsa::EventKind::WillExecute { .. } = event.kind {
                logs.lock()
                    .unwrap()
                    .push(format!("Event: {:?}", event.debug(self)));
            }
        }
    }
}

// impl salsa::ParallelDatabase for RootDatabase {
//     fn snapshot(&self) -> salsa::Snapshot<Self> {
//         salsa::Snapshot::new(RootDatabase {
//             storage: self.storage.snapshot(),
//             logs: self.logs.clone(),
//         })
//     }
// }

pub trait Db: salsa::DbWithJar<Jar> {
    fn input(&self, path: &str) -> Source;
}

impl Db for RootDatabase {
    fn input(&self, path: &str) -> Source {
        Source::new(self, path.into())
    }
}
