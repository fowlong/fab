//! Placeholder for future TrueType font subsetting logic.

use anyhow::Result;

#[allow(unused_variables)]
pub fn subset_font(font_bytes: &[u8], glyph_ids: &[u16]) -> Result<Vec<u8>> {
    // TODO: integrate ttf-parser + custom writer.
    let _ = (font_bytes, glyph_ids);
    Ok(Vec::new())
}
