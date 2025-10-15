use std::collections::HashMap;

use parking_lot::RwLock;
use uuid::Uuid;

use crate::types::DocumentIR;

#[derive(Debug, Clone)]
pub struct DocumentEntry {
    pub id: String,
    pub original_pdf: Vec<u8>,
    pub current_pdf: Vec<u8>,
    pub ir: DocumentIR,
}

impl DocumentEntry {
    pub fn new(original_pdf: Vec<u8>, ir: DocumentIR) -> Self {
        let id = Uuid::new_v4().to_string();
        Self {
            id: id.clone(),
            current_pdf: original_pdf.clone(),
            original_pdf,
            ir,
        }
    }
}

#[derive(Debug, Default)]
pub struct DocumentStore {
    inner: RwLock<HashMap<String, DocumentEntry>>,
}

impl DocumentStore {
    pub fn new() -> Self {
        Self { inner: RwLock::new(HashMap::new()) }
    }

    pub fn insert(&self, entry: DocumentEntry) -> String {
        let id = entry.id.clone();
        self.inner.write().insert(id.clone(), entry);
        id
    }

    pub fn get(&self, id: &str) -> Option<DocumentEntry> {
        self.inner.read().get(id).cloned()
    }

    pub fn update_pdf(&self, id: &str, pdf: Vec<u8>) {
        if let Some(entry) = self.inner.write().get_mut(id) {
            entry.current_pdf = pdf;
        }
    }

    pub fn update_ir(&self, id: &str, ir: DocumentIR) {
        if let Some(entry) = self.inner.write().get_mut(id) {
            entry.ir = ir;
        }
    }
}
