use anyhow::Result;

/// Placeholder loader that currently keeps the provided bytes as-is.
pub fn load_pdf(bytes: Vec<u8>) -> Result<Vec<u8>> {
    Ok(bytes)
}
