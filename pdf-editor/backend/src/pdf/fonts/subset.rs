//! Minimal font subsetting placeholder.

pub fn subset_font(_font_data: &[u8], _glyphs: &[u32]) -> Vec<u8> {
    // A real implementation would create a new font with just the requested glyphs.
    Vec::new()
}
