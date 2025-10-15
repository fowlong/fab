use std::collections::HashMap;

use parking_lot::RwLock;
use uuid::Uuid;

use crate::types::{DocumentIR, PatchOperation};

use super::{extract, patch};

#[derive(Default)]
pub struct DocumentStore {
    inner: RwLock<HashMap<String, DocumentEntry>>,
}

pub struct DocumentEntry {
    original: Vec<u8>,
    current: Vec<u8>,
    ir: DocumentIR,
}

impl DocumentStore {
    pub async fn insert(&self, bytes: Vec<u8>) -> anyhow::Result<String> {
        let ir = extract::extract_ir(&bytes)?;
        let id = Uuid::new_v4().to_string();
        let entry = DocumentEntry {
            original: bytes.clone(),
            current: bytes,
            ir,
        };
        self.inner.write().insert(id.clone(), entry);
        Ok(id)
    }

    pub async fn ir(&self, doc_id: &str) -> anyhow::Result<DocumentIR> {
        let guard = self.inner.read();
        let entry = guard
            .get(doc_id)
            .ok_or_else(|| anyhow::anyhow!("unknown document"))?;
        Ok(entry.ir.clone())
    }

    pub async fn latest_pdf(&self, doc_id: &str) -> anyhow::Result<Vec<u8>> {
        let guard = self.inner.read();
        let entry = guard
            .get(doc_id)
            .ok_or_else(|| anyhow::anyhow!("unknown document"))?;
        Ok(entry.current.clone())
    }

    pub async fn apply_patch(
        &self,
        doc_id: &str,
        operations: Vec<PatchOperation>,
    ) -> anyhow::Result<patch::PatchResult> {
        let mut guard = self.inner.write();
        let entry = guard
            .get_mut(doc_id)
            .ok_or_else(|| anyhow::anyhow!("unknown document"))?;
        let result = patch::apply_patch(entry, operations)?;
        Ok(result)
    }
}

pub struct PatchContext<'a> {
    pub entry: &'a mut DocumentEntry,
}

impl DocumentEntry {
    pub fn ir_mut(&mut self) -> &mut DocumentIR {
        &mut self.ir
    }

    pub fn current_pdf(&self) -> &[u8] {
        &self.current
    }

    pub fn update_pdf(&mut self, new_pdf: Vec<u8>) {
        self.current = new_pdf;
    }
}

impl<'a> From<&'a mut DocumentEntry> for PatchContext<'a> {
    fn from(entry: &'a mut DocumentEntry) -> Self {
        Self { entry }
    }
}
