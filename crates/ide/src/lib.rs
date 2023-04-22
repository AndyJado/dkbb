use std::sync::{Arc, Mutex};

use ide_db::RootDatabase;

#[derive(Debug)]
pub struct AnalysisHost {
    db: Arc<Mutex<RootDatabase>>,
}

impl AnalysisHost {
    pub fn new() -> AnalysisHost {
        AnalysisHost {
            db: Arc::new(Mutex::new(RootDatabase::new())),
        }
    }
}
