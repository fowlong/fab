use std::collections::HashMap;

use parking_lot::RwLock;
use uuid::Uuid;

use crate::{
    pdf::{extract, patch},
    types::{DocumentIR, PatchOperation, PatchResponse},
};

#[derive(Default)]
pub struct DocumentStore {
    docs: RwLock<HashMap<Uuid, DocumentRecord>>,
}

pub struct DocumentRecord {
    pub doc_id: Uuid,
    pub filename: Option<String>,
    pub original_pdf: Vec<u8>,
    pub current_pdf: Vec<u8>,
    pub ir: Option<DocumentIR>,
    pub revision: u64,
}

pub struct InsertResult {
    pub doc_id: Uuid,
}

impl DocumentStore {
    pub fn insert(&self, bytes: Vec<u8>, filename: Option<String>) -> InsertResult {
        let doc_id = Uuid::new_v4();
        let record = DocumentRecord {
            doc_id,
            filename,
            original_pdf: bytes.clone(),
            current_pdf: bytes,
            ir: None,
            revision: 0,
        };
        self.docs.write().insert(doc_id, record);
        InsertResult { doc_id }
    }

    pub fn get_or_extract_ir(&self, doc_id: &Uuid) -> Option<DocumentIR> {
        let mut docs = self.docs.write();
        let record = docs.get_mut(doc_id)?;
        if let Some(ir) = &record.ir {
            return Some(ir.clone());
        }
        let ir = extract::extract_ir(&record.current_pdf).ok()?;
        record.ir = Some(ir.clone());
        Some(ir)
    }

    pub fn apply_patch(&self, doc_id: &Uuid, ops: Vec<PatchOperation>) -> Option<PatchResponse> {
        let mut docs = self.docs.write();
        let record = docs.get_mut(doc_id)?;
        let response = patch::apply_patch(record, ops).ok()?;
        if let Some(ir) = &response.remap {
            // placeholder for IR refresh; currently keep cached IR for compatibility
            let _ = ir;
        }
        if record.ir.is_some() {
            // For now refresh IR on every change until real incremental diff exists
            if let Ok(new_ir) = extract::extract_ir(&record.current_pdf) {
                record.ir = Some(new_ir);
            }
        }
        Some(response)
    }

    pub fn get_pdf(&self, doc_id: &Uuid) -> Option<Vec<u8>> {
        self.docs
            .read()
            .get(doc_id)
            .map(|record| record.current_pdf.clone())
    }
}
