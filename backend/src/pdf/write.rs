use anyhow::{Context, Result};
use lopdf::{Dictionary, Document, IncrementalDocument, Object, ObjectId, Stream};

pub fn incremental_update(
    original_bytes: &[u8],
    document: &Document,
    page_id: ObjectId,
    new_content: &[u8],
) -> Result<Vec<u8>> {
    let mut incremental =
        IncrementalDocument::create_from(original_bytes.to_vec(), document.clone());

    let new_stream_id = (incremental.new_document.max_id + 1, 0);
    incremental.new_document.max_id += 1;

    let mut stream_dict = Dictionary::new();
    stream_dict.set("Length", Object::Integer(new_content.len() as i64));
    let stream = Stream::new(stream_dict, new_content.to_vec());
    incremental
        .new_document
        .objects
        .insert(new_stream_id, Object::Stream(stream));

    let mut page_dict = document
        .get_dictionary(page_id)
        .context("page dictionary missing during incremental update")?
        .clone();
    page_dict.set("Contents", Object::Reference(new_stream_id));
    incremental
        .new_document
        .objects
        .insert(page_id, Object::Dictionary(page_dict));

    let new_size = incremental.new_document.max_id + 1;
    incremental.new_document.reference_table.size = new_size;
    incremental
        .new_document
        .trailer
        .set("Size", Object::Integer(new_size as i64));

    let mut output = Vec::new();
    incremental.save_to(&mut output)?;
    Ok(output)
}
