//! Incremental PDF writer utilities.

use anyhow::{Context, Result};
use lopdf::{Document, IncrementalDocument, Object, ObjectId, Stream};

pub fn incremental_update(
    original_document: &Document,
    original_bytes: &[u8],
    page_id: ObjectId,
    new_stream_content: Vec<u8>,
) -> Result<(Vec<u8>, Document)> {
    let mut incremental =
        IncrementalDocument::create_from(original_bytes.to_vec(), original_document.clone());

    let mut stream = Stream::new(lopdf::Dictionary::new(), Vec::new());
    stream.set_plain_content(new_stream_content);
    let stream_id = incremental.new_document.add_object(stream);

    let mut page_dict = original_document
        .get_dictionary(page_id)
        .context("failed to resolve page dictionary")?
        .clone();
    page_dict.set("Contents", Object::Reference(stream_id));
    incremental
        .new_document
        .set_object(page_id, Object::Dictionary(page_dict));

    let mut buffer = Vec::new();
    incremental.save_to(&mut buffer)?;
    let updated_document = Document::load_mem(&buffer)?;
    Ok((buffer, updated_document))
}
