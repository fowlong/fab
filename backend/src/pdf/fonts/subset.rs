//! Font subsetting stub.

use anyhow::Result;

pub fn subset_font(_font_data: &[u8], _glyphs: &[u32]) -> Result<Vec<u8>> {
    Ok(Vec::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subset_font_returns_empty_placeholder() {
        let subset = subset_font(&[0u8; 4], &[10, 20]).expect("subset should succeed");
        assert!(subset.is_empty());
    }
}
