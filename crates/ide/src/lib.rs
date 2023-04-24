use std::sync::{Arc, Mutex};

use ide_db::RootDatabase;

#[derive(Debug)]
pub struct AnalysisHost {
    pub db: Arc<Mutex<RootDatabase>>,
}

impl AnalysisHost {
    pub fn db(&self) -> std::sync::MutexGuard<'_, RootDatabase> {
        self.db.lock().unwrap()
    }
    pub fn db_with(&self, f: &dyn Fn(&RootDatabase)) {
        f(&*self.db.lock().unwrap())
    }
    pub fn new() -> AnalysisHost {
        AnalysisHost {
            db: Arc::new(Mutex::new(RootDatabase::new())),
        }
    }
}
