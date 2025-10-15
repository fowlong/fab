//! Incremental PDF writer utilities.

use anyhow::Result;
use lopdf::{Dictionary, Document, IncrementalDocument, Object, Stream};

pub fn incremental_update(
    previous_bytes: Vec<u8>,
    previous_doc: Document,
    page_id: lopdf::ObjectId,
    stream_bytes: Vec<u8>,
) -> Result<(Vec<u8>, Document)> {
    let mut incremental = IncrementalDocument::create_from(previous_bytes, previous_doc);
    let stream_id = incremental
        .new_document
        .add_object(Stream::new(Dictionary::new(), stream_bytes.clone()));

    incremental.opt_clone_object_to_new_document(page_id)?;
    {
        let page = incremental
            .new_document
            .get_object_mut(page_id)
            .and_then(Object::as_dict_mut)?;
        page.set("Contents", Object::Reference(stream_id));
    }

    let mut buffer = Vec::new();
    incremental.save_to(&mut buffer)?;
    let updated_document = Document::load_mem(&buffer)?;
    Ok((buffer, updated_document))
}
