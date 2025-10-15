use anyhow::Result;

/// Placeholder for incremental update logic. The current implementation simply
/// returns the provided bytes unchanged so the HTTP API can already stream a
/// PDF response to the frontend.
pub fn incremental_update(bytes: Vec<u8>) -> Result<Vec<u8>> {
    Ok(bytes)
}
