//! Utilities for loading and interrogating PDF documents.

use anyhow::{anyhow, Result};
use lopdf::{Document, ObjectId, Stream};

pub fn parse_document(bytes: &[u8]) -> Result<Document> {
    let mut document = Document::load_mem(bytes)?;
    document.version = document.version; // keep declared version intact
    Ok(document)
}

pub fn get_page_zero(document: &Document) -> Result<(ObjectId, &lopdf::Dictionary)> {
    let pages = document.get_pages();
    let (&page_id, _) = pages
        .iter()
        .next()
        .ok_or_else(|| anyhow!("Document has no pages"))?;
    let object = document
        .get_object(page_id)?
        .as_dict()
        .map_err(|_| anyhow!("Page object is not a dictionary"))?;
    Ok((page_id, object))
}

pub fn resolve_contents(document: &Document, page_dict: &lopdf::Dictionary) -> Result<Vec<(ObjectId, Vec<u8>)>> {
    use lopdf::Object::*;

    let contents = page_dict
        .get(b"Contents")
        .ok_or_else(|| anyhow!("Page is missing /Contents"))?;

    let mut streams = Vec::new();
    match contents {
        Array(items) => {
            for item in items {
                let reference = item
                    .as_reference()
                    .ok_or_else(|| anyhow!("Content array item is not a reference"))?;
                let bytes = load_stream_bytes(document, *reference)?;
                streams.push((*reference, bytes));
            }
        }
        Reference(reference) => {
            let bytes = load_stream_bytes(document, *reference)?;
            streams.push((*reference, bytes));
        }
        Stream(stream) => {
            let bytes = stream.decode_content()?;
            // Streams embedded inline have a synthetic object id of (0, 0)
            streams.push(((0, 0).into(), bytes));
        }
        _ => return Err(anyhow!("Unsupported /Contents type")),
    }

    Ok(streams)
}

pub fn load_stream_bytes(document: &Document, object_id: ObjectId) -> Result<Vec<u8>> {
    let object = document
        .get_object(object_id)?
        .as_stream()
        .map_err(|_| anyhow!("Content reference did not resolve to a stream"))?;
    object.decode_content()
}

pub fn create_stream(content: Vec<u8>) -> Stream {
    let mut stream = Stream::new(lopdf::Dictionary::new(), content);
    stream.compress = false;
    stream
}

pub fn next_object_id(document: &Document) -> u32 {
    document
        .objects
        .keys()
        .map(|id| id.0)
        .max()
        .unwrap_or(0)
        + 1
}
