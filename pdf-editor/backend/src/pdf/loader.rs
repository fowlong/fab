use parking_lot::Mutex;
use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;

pub struct PdfLoader {
    cache: Mutex<HashMap<Uuid, PathBuf>>,
}

impl PdfLoader {
    pub fn new() -> Self {
        Self {
            cache: Mutex::new(HashMap::new()),
        }
    }

    pub fn register(&self, id: Uuid, path: PathBuf) {
        self.cache.lock().insert(id, path);
    }
}
