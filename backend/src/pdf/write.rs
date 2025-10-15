use anyhow::Result;
use lopdf::Document;

pub fn incremental_save(_doc: &Document) -> Result<Vec<u8>> {
    // Placeholder for real incremental save logic.
    let mut buffer = Vec::new();
    _doc.save_to(&mut buffer)?;
    Ok(buffer)
}
