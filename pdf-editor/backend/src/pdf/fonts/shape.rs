pub struct ShapedGlyph {
  pub gid: u32,
  pub advance: f32,
  pub offset_x: f32,
  pub offset_y: f32,
}

/// Shapes text using Harfbuzz (stub implementation for scaffolding).
/// The MVP currently returns a simple left-to-right sequence with
/// monotonically increasing advances based on font size.
pub fn shape_text(_font_data: &[u8], text: &str, size: f32) -> anyhow::Result<Vec<ShapedGlyph>> {
  let mut advance = 0.0_f32;
  let mut shaped = Vec::with_capacity(text.chars().count());
  for ch in text.chars() {
    let glyph = ShapedGlyph {
      gid: ch as u32,
      advance,
      offset_x: 0.0,
      offset_y: 0.0,
    };
    shaped.push(glyph);
    advance += size * 0.6;
  }
  Ok(shaped)
}
