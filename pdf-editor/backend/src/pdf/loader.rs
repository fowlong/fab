use std::{collections::HashMap, sync::Arc};

use bytes::Bytes;
use tokio::sync::RwLock;

use crate::types::PdfState;

#[derive(Clone)]
pub struct StoredDocument {
    pub pdf_bytes: Bytes,
    pub ir: Arc<PdfState>,
}

#[derive(Default)]
pub struct PdfDocumentStore {
    inner: RwLock<HashMap<String, StoredDocument>>,
}

impl PdfDocumentStore {
    pub async fn store(&self, doc_id: String, pdf_bytes: Vec<u8>, ir: PdfState) {
        let mut guard = self.inner.write().await;
        guard.insert(
            doc_id,
            StoredDocument {
                pdf_bytes: Bytes::from(pdf_bytes),
                ir: Arc::new(ir),
            },
        );
    }

    pub async fn get(&self, doc_id: &str) -> Option<StoredDocument> {
        let guard = self.inner.read().await;
        guard.get(doc_id).cloned()
    }
}
