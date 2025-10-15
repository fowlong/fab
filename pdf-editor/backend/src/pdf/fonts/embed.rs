//! Placeholder font embedding utilities.

use super::subset::SubsetFont;

/// Represents the PDF objects required to embed a font.
#[derive(Debug, Clone)]
pub struct EmbeddedFont {
    pub font_dict: serde_json::Value,
    pub font_file: Vec<u8>,
}

#[must_use]
pub fn embed_subset_font(subset: &SubsetFont) -> EmbeddedFont {
    EmbeddedFont {
        font_dict: serde_json::json!({
            "Type": "Font",
            "Subtype": "Type0",
            "BaseFont": subset.name,
        }),
        font_file: subset.data.clone(),
    }
}
