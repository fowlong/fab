use anyhow::Result;

pub struct ShapedGlyph {
  pub glyph_id: u32,
  pub advance: f32,
}

pub fn shape_text(_font_data: &[u8], _text: &str) -> Result<Vec<ShapedGlyph>> {
  // Placeholder shaping.
  Ok(vec![])
}
