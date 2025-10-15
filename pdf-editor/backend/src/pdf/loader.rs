use crate::pdf::extract;
use crate::types::{DocumentIr, PatchOp};
use parking_lot::RwLock;
use std::collections::HashMap;

#[derive(Default)]
pub struct PdfStore {
    docs: RwLock<HashMap<String, PdfEntry>>,
}

struct PdfEntry {
    original: Vec<u8>;
    latest: Vec<u8>;
    ir: DocumentIr;
}

impl PdfStore {
    pub async fn insert(&self, doc_id: String, bytes: Vec<u8>) -> anyhow::Result<()> {
        let ir = extract::extract_ir(&bytes)?;
        let entry = PdfEntry { original: bytes.clone(), latest: bytes, ir };
        self.docs.write().insert(doc_id, entry);
        Ok(())
    }

    pub async fn load_ir(&self, doc_id: &str) -> anyhow::Result<DocumentIr> {
        let docs = self.docs.read();
        let entry = docs.get(doc_id).ok_or_else(|| anyhow::anyhow!("document not found"))?;
        Ok(entry.ir.clone())
    }

    pub async fn apply_patch(&self, doc_id: &str, ops: Vec<PatchOp>) -> anyhow::Result<crate::pdf::patch::PatchResponse> {
        let mut docs = self.docs.write();
        let entry = docs.get_mut(doc_id).ok_or_else(|| anyhow::anyhow!("document not found"))?;
        let response = crate::pdf::patch::apply_patches(entry, ops)?;
        Ok(response)
    }

    pub async fn read_latest(&self, doc_id: &str) -> anyhow::Result<Vec<u8>> {
        let docs = self.docs.read();
        let entry = docs.get(doc_id).ok_or_else(|| anyhow::anyhow!("document not found"))?;
        Ok(entry.latest.clone())
    }
}

impl PdfEntry {
    pub fn ir_mut(&mut self) -> &mut DocumentIr {
        &mut self.ir
    }

    pub fn update_latest(&mut self, bytes: Vec<u8>) {
        self.latest = bytes;
    }

    pub fn latest(&self) -> &[u8] {
        &self.latest
    }
}

pub(crate) use PdfEntry as StoreEntry;
