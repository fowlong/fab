use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::types::{DocumentId, PatchOperation, PatchResponse};

#[derive(Default)]
pub struct PatchEngine {
    state: Arc<RwLock<HashMap<DocumentId, PatchHistory>>>,
}

#[derive(Default)]
struct PatchHistory {
    pub operations: Vec<PatchOperation>;
}

impl PatchEngine {
    pub async fn apply(&self, doc_id: &DocumentId, ops: Vec<PatchOperation>) -> Result<PatchResponse> {
        let mut state = self.state.write().await;
        let entry = state.entry(doc_id.clone()).or_default();
        entry.operations.extend(ops);
        Ok(PatchResponse {
            ok: true,
            updated_pdf: None,
        })
    }
}
