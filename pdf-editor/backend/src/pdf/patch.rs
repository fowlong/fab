use std::collections::HashMap;

use anyhow::Result;

use crate::{
    pdf::{loader::StoredDocument, write},
    types::{PatchOp, RefUpdate},
};

#[derive(Debug, Clone)]
pub struct PatchOutcome {
    pub updated_pdf: Vec<u8>,
    pub remap: HashMap<String, RefUpdate>,
}

pub fn apply_patch(doc: &mut StoredDocument, _ops: &[PatchOp]) -> Result<PatchOutcome> {
    let increment = write::append_incremental_update(doc)?;
    doc.set_latest_bytes(increment.bytes.clone());

    Ok(PatchOutcome {
        updated_pdf: increment.bytes,
        remap: HashMap::new(),
    })
}
