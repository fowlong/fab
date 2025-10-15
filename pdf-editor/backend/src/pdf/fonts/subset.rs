//! Font subsetting scaffolding. The MVP returns the original font bytes
//! unchanged but documents the intended API surface.

use anyhow::Result;

pub struct SubsetResult {
    pub font_bytes: Vec<u8>,
    pub glyph_map: Vec<u32>,
}

pub fn subset_font(font_data: &[u8], glyphs: &[u32]) -> Result<SubsetResult> {
    Ok(SubsetResult {
        font_bytes: font_data.to_vec(),
        glyph_map: glyphs.to_vec(),
    })
}
