use anyhow::Result;

pub fn incremental_update(current: &[u8], _changes: &[u8]) -> Result<Vec<u8>> {
    Ok(current.to_vec())
}
