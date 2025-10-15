use std::collections::HashMap;
use std::sync::Arc;

use anyhow::{anyhow, Context, Result};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::pdf::extract;
use crate::types::DocumentIr;

#[derive(Debug, Default, Clone)]
pub struct DocumentState {
    pub original_pdf: Arc<Vec<u8>>,
    pub latest_pdf: Arc<Vec<u8>>,
    pub ir: DocumentIr,
}

#[derive(Debug, Default)]
pub struct DocumentStore {
    inner: RwLock<HashMap<Uuid, DocumentState>>,
}

impl DocumentStore {
    pub fn new() -> Self {
        Self {
            inner: RwLock::new(HashMap::new()),
        }
    }

    pub async fn insert(&self, doc_id: Uuid, state: DocumentState) {
        let mut map = self.inner.write().await;
        map.insert(doc_id, state);
    }

    pub async fn get(&self, doc_id: &Uuid) -> Option<DocumentState> {
        let map = self.inner.read().await;
        map.get(doc_id).cloned()
    }

    pub async fn update_pdf(
        &self,
        doc_id: &Uuid,
        updated_pdf: Vec<u8>,
        ir: DocumentIr,
    ) -> Result<()> {
        let mut map = self.inner.write().await;
        let entry = map
            .get_mut(doc_id)
            .ok_or_else(|| anyhow!("document not found"))?;
        entry.latest_pdf = Arc::new(updated_pdf);
        entry.ir = ir;
        Ok(())
    }
}

pub async fn load_document(bytes: Vec<u8>) -> Result<DocumentState> {
    let ir = extract::build_initial_ir(&bytes)
        .await
        .context("build IR")?;
    let shared = Arc::new(bytes);
    Ok(DocumentState {
        original_pdf: shared.clone(),
        latest_pdf: shared,
        ir,
    })
}
