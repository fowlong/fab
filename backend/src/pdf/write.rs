use anyhow::Result;
use lopdf::{dictionary, IncrementalDocument, Object, ObjectId, Stream};

pub fn incremental_update(
    document: &lopdf::Document,
    original_bytes: &[u8],
    page_id: ObjectId,
    content_bytes: &[u8],
) -> Result<Vec<u8>> {
    let mut incremental =
        IncrementalDocument::create_from(original_bytes.to_vec(), document.clone());
    let stream = Stream::new(dictionary! {}, content_bytes.to_vec());
    let content_id = incremental.new_document.add_object(stream);
    incremental.opt_clone_object_to_new_document(page_id)?;
    {
        let page_dict = incremental
            .new_document
            .get_object_mut(page_id)
            .and_then(Object::as_dict_mut)?;
        page_dict.set("Contents", Object::Reference(content_id));
    }
    let mut buffer = Vec::new();
    incremental.save_to(&mut buffer)?;
    Ok(buffer)
}
