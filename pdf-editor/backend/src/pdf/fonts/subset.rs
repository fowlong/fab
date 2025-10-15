//! Minimal TrueType subsetter utilities.

use anyhow::Result;

pub struct SubsetResult {
    pub font_name: String,
    pub subset_data: Vec<u8>,
}

pub fn subset_font(_font_data: &[u8], _codepoints: &[u32]) -> Result<SubsetResult> {
    // TODO: subset the TrueType font to the requested codepoints.
    Ok(SubsetResult {
        font_name: "SubsetFont".to_string(),
        subset_data: Vec::new(),
    })
}
