use std::collections::HashMap;

use serde_json::Value;

use super::{loader::PatchContext, write};
use crate::types::PatchOperation;

pub struct PatchResult {
    pub ok: bool,
    pub updated_pdf: Option<String>,
    pub remap: HashMap<String, Value>,
}

pub fn apply_patch(
    entry: &mut super::loader::DocumentEntry,
    operations: Vec<PatchOperation>,
) -> anyhow::Result<PatchResult> {
    if operations.is_empty() {
        return Ok(PatchResult {
            ok: true,
            updated_pdf: None,
            remap: HashMap::new(),
        });
    }

    let mut context = PatchContext::from(entry);
    write::update_ir(context.entry.ir_mut());
    let updated_pdf = write::incremental_save(&mut context)?;

    Ok(PatchResult {
        ok: true,
        updated_pdf: Some(updated_pdf),
        remap: HashMap::new(),
    })
}
