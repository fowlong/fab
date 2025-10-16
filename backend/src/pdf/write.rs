//! Incremental PDF writer utilities.

use anyhow::Result;
use lopdf::{Document, IncrementalDocument, ObjectId};

pub struct StreamUpdate {
    pub object_id: ObjectId,
    pub content: Vec<u8>,
}

pub fn incremental_update(
    previous_bytes: &[u8],
    document: &Document,
    updates: &[StreamUpdate],
) -> Result<(Vec<u8>, Document)> {
    let mut incremental =
        IncrementalDocument::create_from(previous_bytes.to_vec(), document.clone());
    for update in updates {
        incremental.opt_clone_object_to_new_document(update.object_id)?;
        let object = incremental
            .new_document
            .get_object_mut(update.object_id)
            .and_then(lopdf::Object::as_stream_mut)?;
        object.set_plain_content(update.content.clone());
    }
    let mut buffer = Vec::new();
    incremental.save_to(&mut buffer)?;
    let updated_document = Document::load_mem(&buffer)?;
    Ok((buffer, updated_document))
}
