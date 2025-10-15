use crate::types::DocumentIR;

pub fn incremental_save(bytes: &[u8], _ir: &DocumentIR) -> anyhow::Result<Vec<u8>> {
    Ok(bytes.to_vec())
}
