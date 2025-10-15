//! Incremental PDF writer for updated content streams.

use anyhow::{anyhow, Result};
use lopdf::{Dictionary, Document, IncrementalDocument, Object, ObjectId, Stream};

pub fn incremental_update(
    original_bytes: &[u8],
    original_doc: &Document,
    page_id: ObjectId,
    new_contents: Vec<u8>,
) -> Result<Vec<u8>> {
    let mut incremental =
        IncrementalDocument::create_from(original_bytes.to_vec(), original_doc.clone());

    let stream_id = incremental.new_document.new_object_id();
    let mut dict = Dictionary::new();
    dict.set("Length", i64::try_from(new_contents.len()).unwrap_or(0));
    let stream = Stream::new(dict, new_contents);
    incremental
        .new_document
        .set_object(stream_id, Object::Stream(stream));

    incremental
        .opt_clone_object_to_new_document(page_id)
        .map_err(|err| anyhow!("failed to clone page dictionary: {err}"))?;

    let page_object = incremental
        .new_document
        .get_object_mut(page_id)
        .map_err(|err| anyhow!("page not found in incremental doc: {err}"))?;
    let page_dict = page_object
        .as_dict_mut()
        .map_err(|_| anyhow!("page object is not a dictionary"))?;
    page_dict.set("Contents", Object::Reference(stream_id));

    let mut buffer = Vec::new();
    incremental
        .save_to(&mut buffer)
        .map_err(|err| anyhow!("failed to write incremental update: {err}"))?;
    Ok(buffer)
}
