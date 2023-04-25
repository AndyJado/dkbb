pub mod helper;
pub mod ir;
pub mod line_index;
use core::fmt;
use std::{
    fs,
    sync::{Arc, Mutex},
};

use ir::SourceProgram;
use line_index::LineIndex;
use salsa::DebugWithDb;
use tower_lsp::lsp_types::{Diagnostic, Range};

#[salsa::jar(db = Db)]
pub struct Jar(
    // input
    crate::ir::SourceProgram,
    // struct
    crate::ir::Diagnostics,
    // fn
    crate::ir::compile,
);

#[derive(Default)]
#[salsa::db(crate::Jar)]
pub struct RootDatabase {
    storage: salsa::Storage<Self>,
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
    fn input(&self, path: &str) -> SourceProgram;
}

impl Db for RootDatabase {
    fn input(&self, path: &str) -> SourceProgram {
        let file = fs::read_to_string(path);
        let f = match file {
            Ok(f) => f,
            Err(_) => String::new(),
        };
        let lines = LineIndex::new(&f);
        SourceProgram::new(self, f, lines)
    }
}
