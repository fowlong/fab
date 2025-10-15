use std::{collections::HashMap, path::PathBuf, sync::Arc};

use anyhow::Context;
use axum::extract::{multipart::Field, Multipart};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::types::{DocumentIr, PatchOperation, PatchResponse};

use super::{extract, patch};

#[derive(Default)]
pub struct PdfStore {
    inner: RwLock<StoreInner>,
}

#[derive(Default)]
struct StoreInner {
    docs: HashMap<String, Arc<RwLock<PdfDocument>>>,
}

struct PdfDocument {
    #[allow(dead_code)]
    original_path: Option<PathBuf>,
    bytes: Vec<u8>,
    ir: DocumentIr,
}

impl PdfStore {
    pub async fn open(&self, mut payload: Multipart) -> anyhow::Result<String> {
        let mut file_bytes = None;
        while let Some(field) = payload.next_field().await? {
            if field.name() == Some("file") {
                file_bytes = Some(read_field(field).await?);
            }
        }
        let bytes = file_bytes.context("missing file field")?;
        let ir = extract::extract_ir(&bytes)?;
        let doc = PdfDocument {
            original_path: None,
            bytes,
            ir,
        };
        let doc_id = Uuid::new_v4().to_string();
        let mut store = self.inner.write().await;
        store
            .docs
            .insert(doc_id.clone(), Arc::new(RwLock::new(doc)));
        Ok(doc_id)
    }

    pub async fn fetch_ir(&self, doc_id: &str) -> anyhow::Result<DocumentIr> {
        let store = self.inner.read().await;
        let doc = store.docs.get(doc_id).context("unknown document")?;
        let doc = doc.read().await;
        Ok(doc.ir.clone())
    }

    pub async fn apply_patch(
        &self,
        doc_id: &str,
        ops: Vec<PatchOperation>,
    ) -> anyhow::Result<PatchResponse> {
        let store = self.inner.read().await;
        let doc = store.docs.get(doc_id).context("unknown document")?;
        let mut doc = doc.write().await;
        let result = patch::apply(&mut doc.ir, &mut doc.bytes, ops)?;
        Ok(result)
    }

    pub async fn download(&self, doc_id: &str) -> anyhow::Result<Vec<u8>> {
        let store = self.inner.read().await;
        let doc = store.docs.get(doc_id).context("unknown document")?;
        let doc = doc.read().await;
        Ok(doc.bytes.clone())
    }
}

async fn read_field(mut field: Field<'_>) -> anyhow::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    while let Some(chunk) = field.chunk().await? {
        bytes.extend_from_slice(&chunk);
    }
    Ok(bytes)
}
