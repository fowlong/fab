use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use bytes::Bytes;
use uuid::Uuid;

use crate::types::DocumentIR;

#[derive(Clone, Default)]
pub struct PdfStore {
    inner: Arc<RwLock<HashMap<Uuid, PdfDocument>>>,
}

pub struct PdfDocument {
    pub original_bytes: Bytes,
    pub current_bytes: Bytes,
    pub ir: DocumentIR,
}

impl PdfStore {
    pub fn insert(&self, id: Uuid, doc: PdfDocument) {
        self.inner.write().unwrap().insert(id, doc);
    }

    pub fn get(&self, id: &Uuid) -> Option<PdfDocument> {
        self.inner.read().unwrap().get(id).cloned()
    }
}

impl Clone for PdfDocument {
    fn clone(&self) -> Self {
        Self {
            original_bytes: self.original_bytes.clone(),
            current_bytes: self.current_bytes.clone(),
            ir: self.ir.clone(),
        }
    }
}
