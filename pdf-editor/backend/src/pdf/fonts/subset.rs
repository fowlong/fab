//! Minimal TrueType subsetting scaffold.

use anyhow::Result;

pub fn subset_font(_font_data: &[u8], _glyph_ids: &[u32]) -> Result<Vec<u8>> {
    // Placeholder implementation that simply returns the source font bytes.
    Ok(Vec::new())
}
