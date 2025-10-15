/// Produces a minimal subset of TrueType font data containing the supplied glyphs.
/// This MVP implementation simply returns the original font bytes.
pub fn subset_font(font_data: &[u8], _glyphs: &[u32]) -> anyhow::Result<Vec<u8>> {
  Ok(font_data.to_vec())
}

/// Generates a deterministic subset font name following PDF conventions.
pub fn subset_font_name(base: &str) -> String {
  let prefix: String = base
    .chars()
    .filter(|c| c.is_ascii_alphabetic())
    .take(6)
    .collect();
  format!("{}+{}", prefix.to_ascii_uppercase(), base)
}
