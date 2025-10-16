//! Incremental PDF writer helpers.

use anyhow::Result;
use lopdf::{Document, IncrementalDocument, Object, ObjectId};

pub fn incremental_update(
    prev_doc: &Document,
    original: &[u8],
    updates: &[(ObjectId, Object)],
) -> Result<Vec<u8>> {
    let mut incremental = IncrementalDocument::create_from(original.to_vec(), prev_doc.clone());
    for (id, object) in updates {
        incremental.new_document.set_object(*id, object.clone());
        if id.0 > incremental.new_document.max_id {
            incremental.new_document.max_id = id.0;
        }
    }
    let mut buffer = Vec::new();
    incremental.save_to(&mut buffer)?;
    Ok(buffer)
}
