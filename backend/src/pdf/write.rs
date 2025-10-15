use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;

use super::loader::PatchContext;
use crate::types::DocumentIR;

pub fn incremental_save(context: &mut PatchContext<'_>) -> anyhow::Result<String> {
    let bytes = context.entry.current_pdf();
    Ok(format!(
        "data:application/pdf;base64,{}",
        BASE64.encode(bytes)
    ))
}

pub fn update_ir(_ir: &mut DocumentIR) {
    // Placeholder: future updates will mutate the IR when patches are applied.
}
