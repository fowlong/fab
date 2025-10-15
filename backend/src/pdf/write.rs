use anyhow::Result;
use lopdf::{Dictionary, Document, IncrementalDocument, Object, ObjectId, Stream};

pub fn incremental_update(
    original: &[u8],
    document: &Document,
    page_id: ObjectId,
    content: Vec<u8>,
) -> Result<Vec<u8>> {
    let mut incremental = IncrementalDocument::create_from(original.to_vec(), document.clone());

    let mut dict = Dictionary::new();
    dict.set("Length", content.len() as i64);
    let stream = Stream {
        dict,
        content,
        allows_compression: true,
        start_position: None,
    };

    let stream_id = incremental.new_document.add_object(stream);
    incremental.opt_clone_object_to_new_document(page_id)?;
    {
        let page_dict = incremental
            .new_document
            .get_object_mut(page_id)
            .and_then(Object::as_dict_mut)?;
        page_dict.set("Contents", Object::Reference(stream_id));
    }

    let mut output = Vec::new();
    incremental.save_to(&mut output)?;
    Ok(output)
}
