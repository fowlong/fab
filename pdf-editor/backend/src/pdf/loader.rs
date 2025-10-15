use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use anyhow::Result;

#[derive(Debug, Clone, Default)]
pub struct DocumentStore {
    inner: Arc<RwLock<HashMap<String, Vec<u8>>>>,
}

impl DocumentStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&self, doc_id: impl Into<String>, bytes: Vec<u8>) {
        let mut guard = self.inner.write().expect("poisoned lock");
        guard.insert(doc_id.into(), bytes);
    }

    pub fn get(&self, doc_id: &str) -> Option<Vec<u8>> {
        let guard = self.inner.read().ok()?;
        guard.get(doc_id).cloned()
    }

    pub fn update(&self, doc_id: &str, bytes: Vec<u8>) -> Result<()> {
        let mut guard = self
            .inner
            .write()
            .map_err(|_| anyhow::anyhow!("lock poisoned"))?;
        guard.insert(doc_id.to_string(), bytes);
        Ok(())
    }
}
