use anyhow::Result;

pub struct SubsetResult {
    pub font_stream: Vec<u8>,
    pub font_name: String,
}

pub fn subset_font(_font_data: &[u8], _glyphs: &[u32]) -> Result<SubsetResult> {
    // Placeholder for TTF subsetting implementation.
    Ok(SubsetResult {
        font_stream: Vec::new(),
        font_name: "Placeholder".into(),
    })
}
