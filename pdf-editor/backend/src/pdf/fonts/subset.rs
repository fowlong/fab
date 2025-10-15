use anyhow::{anyhow, Result};

pub fn subset_font(_glyphs: &[u16]) -> Result<Vec<u8>> {
    Err(anyhow!("font subsetting not yet implemented"))
}
