use super::PdfError;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone, Default)]
pub struct PdfStore {
    inner: Arc<RwLock<HashMap<String, StoredPdf>>>,
}

#[derive(Clone)]
pub struct StoredPdf {
    pub bytes: Vec<u8>,
}

impl PdfStore {
    pub async fn insert(&self, bytes: Vec<u8>) -> String {
        let id = Uuid::new_v4().to_string();
        let pdf = StoredPdf { bytes };
        self.inner.write().insert(id.clone(), pdf);
        id
    }

    pub async fn get(&self, doc_id: &str) -> Result<StoredPdf, PdfError> {
        self.inner
            .read()
            .get(doc_id)
            .cloned()
            .ok_or_else(|| PdfError::Invalid(format!("unknown doc id {doc_id}")))
    }

    pub async fn replace(&self, doc_id: String, pdf: StoredPdf) {
        self.inner.write().insert(doc_id, pdf);
    }
}
