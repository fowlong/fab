//! Incremental writer utilities.

use anyhow::{Context, Result};
use lopdf::{Dictionary, Document, IncrementalDocument, Object, ObjectId, Stream};

/// Append a new revision that replaces the target content stream and returns the updated PDF.
pub fn append_revision(
    current_bytes: &[u8],
    document: &Document,
    page_id: ObjectId,
    target_stream: ObjectId,
    new_stream_bytes: Vec<u8>,
) -> Result<(Vec<u8>, Document)> {
    let mut incremental =
        IncrementalDocument::create_from(current_bytes.to_vec(), document.clone());

    let new_obj_number = incremental.new_document.max_id + 1;
    let new_stream_id = (new_obj_number, 0);
    incremental.new_document.max_id = new_obj_number;

    let stream = Stream::new(Dictionary::new(), new_stream_bytes);
    incremental
        .new_document
        .objects
        .insert(new_stream_id, Object::Stream(stream));

    let new_contents = build_contents_object(document, page_id, target_stream, new_stream_id)?;

    incremental.opt_clone_object_to_new_document(page_id)?;
    let page_dict = incremental
        .new_document
        .get_object_mut(page_id)
        .and_then(Object::as_dict_mut)?;
    page_dict.set("Contents", new_contents);

    let mut buffer = Vec::new();
    incremental.save_to(&mut buffer)?;

    let updated_document = Document::load_mem(&buffer)?;
    Ok((buffer, updated_document))
}

fn build_contents_object(
    document: &Document,
    page_id: ObjectId,
    target_stream: ObjectId,
    new_stream: ObjectId,
) -> Result<Object> {
    let page_dict = document
        .get_object(page_id)
        .context("failed to load page object")?
        .as_dict()
        .context("page object is not a dictionary")?;
    if let Some(contents) = page_dict.get(b"Contents") {
        match contents {
            Object::Reference(existing) => {
                if *existing == target_stream {
                    Ok(Object::Reference(new_stream))
                } else {
                    Ok(Object::Array(vec![
                        Object::Reference(*existing),
                        Object::Reference(new_stream),
                    ]))
                }
            }
            Object::Array(items) => {
                let mut replaced = false;
                let mut array = Vec::with_capacity(items.len());
                for item in items {
                    if let Object::Reference(id) = item {
                        if *id == target_stream {
                            array.push(Object::Reference(new_stream));
                            replaced = true;
                        } else {
                            array.push(Object::Reference(*id));
                        }
                    } else {
                        array.push(item.clone());
                    }
                }
                if !replaced {
                    array.push(Object::Reference(new_stream));
                }
                Ok(Object::Array(array))
            }
            _ => Ok(Object::Reference(new_stream)),
        }
    } else {
        Ok(Object::Reference(new_stream))
    }
}
