use anyhow::Result;

#[derive(Debug, Clone)]
pub struct ShapedGlyph {
    pub glyph_id: u32,
    pub advance_x: f32,
    pub advance_y: f32,
}

pub fn shape_text(_text: &str, _font_data: &[u8]) -> Result<Vec<ShapedGlyph>> {
    Ok(Vec::new())
}
