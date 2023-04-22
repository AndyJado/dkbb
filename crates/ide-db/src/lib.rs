use std::sync::{Arc, Mutex};

use salsa::DebugWithDb;

#[salsa::jar(db = Db)]
pub struct Jar();

#[derive(Default)]
#[salsa::db(crate::Jar)]
pub struct RootDatabase {
    storage: salsa::Storage<Self>,
    logs: Option<Arc<Mutex<Vec<String>>>>,
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

impl salsa::ParallelDatabase for RootDatabase {
    fn snapshot(&self) -> salsa::Snapshot<Self> {
        salsa::Snapshot::new(RootDatabase {
            storage: self.storage.snapshot(),
            logs: self.logs.clone(),
        })
    }
}

pub trait Db: salsa::DbWithJar<Jar> {}

impl<DB> Db for DB where DB: ?Sized + salsa::DbWithJar<Jar> {}
