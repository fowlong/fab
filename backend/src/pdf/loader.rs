use anyhow::Result;
use lopdf::Document;

pub fn load_document_from_bytes(bytes: &[u8]) -> Result<Document> {
    let mut cursor = std::io::Cursor::new(bytes);
    let doc = Document::load_from(&mut cursor)?;
    Ok(doc)
}
