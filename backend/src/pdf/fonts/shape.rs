//! HarfBuzz shaping stub.

use anyhow::Result;

use crate::types::TextGlyph;

pub fn shape_text(_text: &str) -> Result<Vec<TextGlyph>> {
    Ok(Vec::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shape_text_returns_empty_vector_for_now() {
        let glyphs = shape_text("Hello").expect("shaping should succeed");
        assert!(glyphs.is_empty());
    }
}
