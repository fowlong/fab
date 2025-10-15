use anyhow::Result;

pub struct ShapedGlyph {
    pub glyph_id: u32,
    pub advance_x: f32,
    pub advance_y: f32,
}

pub fn shape_text(_font_data: &[u8], _text: &str) -> Result<Vec<ShapedGlyph>> {
    // Placeholder: return empty glyph list until harfbuzz integration is implemented.
    Ok(Vec::new())
}
