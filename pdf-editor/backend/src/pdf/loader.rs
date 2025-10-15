use anyhow::{anyhow, Result};
use bytes::Bytes;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::types::{DocumentId, DocumentIR};

#[derive(Default)]
pub struct PdfLoader {
    documents: Arc<RwLock<HashMap<DocumentId, StoredDocument>>>,
}

#[derive(Default)]
struct StoredDocument {
    pub ir: Option<DocumentIR>;
    pub bytes: Option<Bytes>;
}

impl PdfLoader {
    pub async fn open_placeholder(&self) -> DocumentId {
        let doc_id = uuid::Uuid::new_v4().to_string();
        self.documents
            .write()
            .await
            .insert(doc_id.clone(), StoredDocument::default());
        doc_id
    }

    pub async fn fetch_ir(&self, doc_id: &DocumentId) -> Result<DocumentIR> {
        let map = self.documents.read().await;
        let stored = map.get(doc_id).ok_or_else(|| anyhow!("document not found"))?;
        stored
            .ir
            .clone()
            .ok_or_else(|| anyhow!("IR not yet generated for document {}", doc_id))
    }

    pub async fn download(&self, doc_id: &DocumentId) -> Result<Bytes> {
        let map = self.documents.read().await;
        let stored = map.get(doc_id).ok_or_else(|| anyhow!("document not found"))?;
        stored
            .bytes
            .clone()
            .ok_or_else(|| anyhow!("PDF bytes not yet available for document {}", doc_id))
    }

    pub async fn update_ir(&self, doc_id: &DocumentId, ir: DocumentIR) -> Result<()> {
        let mut map = self.documents.write().await;
        let stored = map.get_mut(doc_id).ok_or_else(|| anyhow!("document not found"))?;
        stored.ir = Some(ir);
        Ok(())
    }

    pub async fn update_bytes(&self, doc_id: &DocumentId, bytes: Bytes) -> Result<()> {
        let mut map = self.documents.write().await;
        let stored = map.get_mut(doc_id).ok_or_else(|| anyhow!("document not found"))?;
        stored.bytes = Some(bytes);
        Ok(())
    }
}

#[derive(Debug, Serialize)]
pub struct PlaceholderIrResponse {
    pub doc_id: DocumentId;
    pub pages: Vec<()>;
}
