use anyhow::Result;

pub fn build_font_resource(_subset_bytes: &[u8]) -> Result<lopdf::Object> {
    // Placeholder: return empty stream.
    Ok(lopdf::Object::Null)
}
