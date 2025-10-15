use std::collections::HashMap;

use parking_lot::RwLock;
use uuid::Uuid;

use crate::types::DocumentIR;

#[derive(Clone)]
pub struct StoredDocument {
    pub original_bytes: Vec<u8>,
    pub current_bytes: Vec<u8>,
    pub ir: DocumentIR,
}

impl StoredDocument {
    pub fn new(bytes: Vec<u8>, ir: DocumentIR) -> Self {
        Self {
            original_bytes: bytes.clone(),
            current_bytes: bytes,
            ir,
        }
    }
}

#[derive(Default)]
pub struct DocumentStore {
    inner: RwLock<HashMap<Uuid, StoredDocument>>,
}

impl DocumentStore {
    pub fn new() -> Self {
        Self {
            inner: RwLock::new(HashMap::new()),
        }
    }

    pub fn insert(&self, doc: StoredDocument) -> Uuid {
        let id = Uuid::new_v4();
        self.inner.write().insert(id, doc);
        id
    }

    pub fn get(&self, id: &Uuid) -> Option<StoredDocument> {
        self.inner.read().get(id).cloned()
    }

    pub fn update(&self, id: &Uuid, doc: StoredDocument) {
        self.inner.write().insert(*id, doc);
    }
}
