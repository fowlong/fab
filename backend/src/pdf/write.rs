//! Incremental PDF writer that appends a new revision with updated
//! content streams.

use anyhow::Result;
use lopdf::{Dictionary, Document, IncrementalDocument, Object, Stream};

use super::extract::PageCache;

/// Append an incremental update that replaces page zero's content stream with
/// the supplied bytes.
pub fn incremental_update(
    document: &Document,
    page: &PageCache,
    original: &[u8],
    new_content: Vec<u8>,
) -> Result<Vec<u8>> {
    let mut incremental = IncrementalDocument::create_from(original.to_vec(), document.clone());
    incremental.new_document.version = document.version.clone();

    incremental.opt_clone_object_to_new_document(page.page_id)?;

    incremental.new_document.max_id += 1;
    let new_stream_id = (incremental.new_document.max_id, 0);
    let stream = Stream::new(Dictionary::new(), new_content);
    incremental
        .new_document
        .objects
        .insert(new_stream_id, Object::Stream(stream));

    let page_object = incremental
        .new_document
        .get_object_mut(page.page_id)?
        .as_dict_mut()?;
    page_object.set("Contents", Object::Reference(new_stream_id));

    let mut buffer = Vec::new();
    incremental.save_to(&mut buffer)?;
    Ok(buffer)
}
