//! Incremental PDF writing helpers.

use anyhow::Result;
use lopdf::{Dictionary, Document, IncrementalDocument, Object, ObjectId, Stream};

pub fn incremental_update(
    previous_doc: &Document,
    original_bytes: &[u8],
    page_id: ObjectId,
    content: Vec<u8>,
) -> Result<(Vec<u8>, Document, ObjectId)> {
    let mut incremental =
        IncrementalDocument::create_from(original_bytes.to_vec(), previous_doc.clone());

    let new_stream_id = incremental.new_document.new_object_id();
    let mut dict = Dictionary::new();
    dict.set("Length", content.len() as i64);
    let mut stream = Stream::new(dict, content);
    stream.allow_compression = false;
    incremental
        .new_document
        .set_object(new_stream_id, Object::Stream(stream));

    incremental.opt_clone_object_to_new_document(page_id)?;
    {
        let page_object = incremental
            .new_document
            .get_object_mut(page_id)
            .and_then(Object::as_dict_mut)?;
        page_object.set("Contents", Object::Reference(new_stream_id));
    }

    let mut buffer = Vec::new();
    incremental.save_to(&mut buffer)?;
    let updated_doc = Document::load_mem(&buffer)?;
    Ok((buffer, updated_doc, new_stream_id))
}
