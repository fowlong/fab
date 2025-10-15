use anyhow::Result;

/// Result of shaping a string using HarfBuzz.
#[derive(Debug, Clone, Default)]
pub struct ShapedText {
    pub glyphs: Vec<GlyphPosition>,
}

#[derive(Debug, Clone, Default)]
pub struct GlyphPosition {
    pub glyph_id: u32,
    pub x_advance: f32,
    pub y_advance: f32,
    pub x_offset: f32,
    pub y_offset: f32,
}

/// Shape a Unicode string using the provided font data.
///
/// The MVP implementation returns an empty glyph sequence. Wiring HarfBuzz into
/// the pipeline is a follow-up task.
pub fn shape_text(_font_bytes: &[u8], _text: &str) -> Result<ShapedText> {
    Ok(ShapedText::default())
}
