//! Utilities for loading and caching PDFs.

use anyhow::Result;
use lopdf::Document;

/// Placeholder loader that simply parses bytes into a `lopdf::Document`.
/// In the future this module will provide caching and sanitisation.
pub fn parse_document(bytes: &[u8]) -> Result<Document> {
    let mut doc = Document::load_mem(bytes)?;
    doc.version = "1.7".to_string();
    Ok(doc)
}

#[cfg(test)]
mod tests {
    use super::*;
    use lopdf::dictionary;

    #[test]
    fn parse_document_sets_version_and_preserves_pages() {
        let mut source = lopdf::Document::with_version("1.0");
        let root_id = lopdf::ObjectId::from((1, 0));
        source
            .trailer
            .set("Root", lopdf::Object::Reference(root_id));
        source.objects.insert(
            root_id,
            lopdf::dictionary! {
                "Type" => lopdf::Object::Name(b"Catalog".to_vec()),
            }
            .into(),
        );
        let mut bytes = Vec::new();
        source
            .save_to(&mut bytes)
            .expect("constructing minimal pdf");

        let document = parse_document(&bytes).expect("generated pdf should parse");

        assert_eq!(document.version, "1.7");
        assert!(document.trailer.get(b"Root").is_ok());
    }
}
