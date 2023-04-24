use std::sync::{Arc, Mutex};

use ide_db::{
    ir::{self, Diagnostic, Diagnostics, SourceProgram},
    RootDatabase,
};

#[derive(Debug)]
pub struct AnalysisHost {
    pub db: Arc<Mutex<RootDatabase>>,
}

impl AnalysisHost {
    pub fn db_with(&self, f: &dyn Fn(&RootDatabase)) {
        f(&*self.db.lock().unwrap())
    }
    pub fn new() -> AnalysisHost {
        AnalysisHost {
            db: Arc::new(Mutex::new(RootDatabase::new())),
        }
    }
}
