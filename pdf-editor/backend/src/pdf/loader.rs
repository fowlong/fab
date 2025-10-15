use parking_lot::RwLock;
use std::{collections::HashMap, sync::Arc};
use uuid::Uuid;

#[derive(Default)]
pub struct DocumentStore {
    documents: RwLock<HashMap<Uuid, Arc<RwLock<StoredDocument>>>>,
}

impl DocumentStore {
    pub fn new() -> Self {
        Self {
            documents: RwLock::new(HashMap::new()),
        }
    }

    pub fn insert(&self, bytes: Vec<u8>) -> Uuid {
        let id = Uuid::new_v4();
        let doc = Arc::new(RwLock::new(StoredDocument::new(bytes)));
        self.documents.write().insert(id, doc);
        id
    }

    pub fn get(&self, id: &Uuid) -> Option<Arc<RwLock<StoredDocument>>> {
        self.documents.read().get(id).cloned()
    }
}

pub struct StoredDocument {
    pub original: Vec<u8>,
    pub current: Vec<u8>,
}

impl StoredDocument {
    fn new(bytes: Vec<u8>) -> Self {
        Self {
            original: bytes.clone(),
            current: bytes,
        }
    }
}
