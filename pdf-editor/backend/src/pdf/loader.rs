use std::{collections::HashMap, path::PathBuf, sync::Arc};

use parking_lot::RwLock;

#[derive(Clone, Default)]
pub struct PdfCache {
    inner: Arc<RwLock<HashMap<PathBuf, Vec<u8>>>>,
}

impl PdfCache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&self, path: PathBuf, bytes: Vec<u8>) {
        self.inner.write().insert(path, bytes);
    }

    pub fn get(&self, path: &PathBuf) -> Option<Vec<u8>> {
        self.inner.read().get(path).cloned()
    }
}
