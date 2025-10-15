use anyhow::Result;

pub fn incremental_update(bytes: &[u8]) -> Result<Vec<u8>> {
    Ok(bytes.to_vec())
}
