//! Incremental PDF writer utilities.

use std::collections::HashMap;

use anyhow::{anyhow, Result};
use lopdf::{Dictionary, Document, IncrementalDocument, Object, ObjectId, Stream};

/// Append an incremental update to the original PDF containing the modified
/// content streams and updated page dictionary.
pub fn incremental_update(
    original_bytes: &[u8],
    previous: &Document,
    page_id: ObjectId,
    contents_order: &[ObjectId],
    contents_is_array: bool,
    updates: &HashMap<ObjectId, Vec<u8>>,
) -> Result<Vec<u8>> {
    if updates.is_empty() {
        return Ok(original_bytes.to_vec());
    }

    let mut incremental =
        IncrementalDocument::create_from(original_bytes.to_vec(), previous.clone());
    incremental.new_document.version = previous.version.clone();

    let mut mapping: HashMap<ObjectId, ObjectId> = HashMap::new();

    for (old_id, data) in updates {
        let new_id = incremental.new_document.new_object_id();
        let mut stream = Stream::new(Dictionary::new(), data.clone());
        stream.set_plain_content(data.clone());
        incremental
            .new_document
            .set_object(new_id, Object::Stream(stream));
        mapping.insert(*old_id, new_id);
    }

    incremental.opt_clone_object_to_new_document(page_id)?;
    let page_object = incremental
        .new_document
        .get_object_mut(page_id)
        .and_then(Object::as_dict_mut)
        .map_err(|_| anyhow!("page dictionary unavailable"))?;

    let mut new_contents: Vec<Object> = Vec::with_capacity(contents_order.len());
    for id in contents_order {
        if let Some(new_id) = mapping.get(id) {
            new_contents.push(Object::Reference(*new_id));
        } else {
            new_contents.push(Object::Reference(*id));
        }
    }

    if new_contents.len() == 1 && !contents_is_array {
        page_object.set("Contents", new_contents.remove(0));
    } else {
        page_object.set("Contents", Object::Array(new_contents));
    }

    let mut buffer = Vec::new();
    incremental.save_to(&mut buffer)?;
    Ok(buffer)
}
