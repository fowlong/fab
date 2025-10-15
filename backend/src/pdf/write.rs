//! Incremental PDF writer utilities.

use anyhow::{anyhow, Result};
use lopdf::{dictionary, Document, IncrementalDocument, Object, Stream};

use super::loader::LoadedDoc;

pub fn incremental_update(doc: &LoadedDoc, content: Vec<u8>) -> Result<(Vec<u8>, Document)> {
    let mut incremental = IncrementalDocument::create_from(doc.bytes.clone(), doc.document.clone());
    incremental.new_document.version = doc.document.version.clone();
    incremental
        .opt_clone_object_to_new_document(doc.page0.page_id)
        .map_err(|err| anyhow!("failed to clone page dictionary: {err}"))?;
    let stream_id = incremental
        .new_document
        .add_object(Stream::new(dictionary! {}, content));
    {
        let page_dict = incremental
            .new_document
            .get_object_mut(doc.page0.page_id)
            .and_then(Object::as_dict_mut)
            .map_err(|_| anyhow!("page dictionary missing"))?;
        page_dict.set("Contents", Object::Reference(stream_id));
    }

    let mut buffer = Vec::new();
    incremental.save_to(&mut buffer)?;
    let updated = Document::load_mem(&buffer)?;
    Ok((buffer, updated))
}
