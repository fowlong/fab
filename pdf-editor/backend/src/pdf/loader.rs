use std::collections::HashMap;
use std::sync::Mutex;
use uuid::Uuid;

use super::extract;
use crate::types::{IntermediateRepresentation, PatchOperation, PatchResponse};

#[derive(Default)]
pub struct DocumentStore {
    inner: Mutex<HashMap<String, DocumentEntry>>,
}

struct DocumentEntry {
    pdf_bytes: Vec<u8>,
    ir: IntermediateRepresentation,
}

impl DocumentStore {
    pub fn open_document_placeholder(&self) -> anyhow::Result<String> {
        let doc_id = Uuid::new_v4().to_string();
        let pdf_bytes = include_bytes!("../../e2e/sample.pdf").to_vec();
        let ir = extract::extract_ir(&pdf_bytes)?;
        let entry = DocumentEntry { pdf_bytes, ir };
        self.inner.lock().unwrap().insert(doc_id.clone(), entry);
        Ok(doc_id)
    }

    pub fn document_ir(&self, doc_id: &str) -> anyhow::Result<IntermediateRepresentation> {
        let guard = self.inner.lock().unwrap();
        let entry = guard
            .get(doc_id)
            .ok_or_else(|| anyhow::anyhow!("Not found"))?;
        Ok(entry.ir.clone())
    }

    pub fn apply_patch(
        &self,
        doc_id: &str,
        _ops: Vec<PatchOperation>,
    ) -> anyhow::Result<PatchResponse> {
        let guard = self.inner.lock().unwrap();
        if guard.get(doc_id).is_none() {
            anyhow::bail!("Not found");
        }
        Ok(PatchResponse::ok())
    }

    pub fn pdf_bytes(&self, doc_id: &str) -> anyhow::Result<Vec<u8>> {
        let guard = self.inner.lock().unwrap();
        let entry = guard
            .get(doc_id)
            .ok_or_else(|| anyhow::anyhow!("Not found"))?;
        Ok(entry.pdf_bytes.clone())
    }
}
