use anyhow::Result;

use super::embed::EmbeddedFont;

#[allow(dead_code)]
pub fn subset_font(_glyphs: &[u32], _font_bytes: &[u8]) -> Result<EmbeddedFont> {
    // Placeholder subsetter. The completed implementation will carve out
    // only the glyph tables required by the edited text.
    Ok(EmbeddedFont::new("SubsetFont", Vec::new()))
}
