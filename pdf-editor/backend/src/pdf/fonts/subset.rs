use anyhow::Result;

/// Minimal representation of a subsetted font.
#[derive(Debug, Clone, Default)]
pub struct FontSubset {
    pub name: String,
    pub bytes: Vec<u8>,
}

/// Placeholder implementation that simply echoes the font data.
pub fn subset_font(font_name: &str, font_bytes: &[u8]) -> Result<FontSubset> {
    Ok(FontSubset {
        name: format!("{font_name}-Subset"),
        bytes: font_bytes.to_vec(),
    })
}
