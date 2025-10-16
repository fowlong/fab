use std::collections::HashMap;

use anyhow::{anyhow, Result};
use lopdf::{Dictionary, IncrementalDocument, Object, Stream};

use super::extract::{PageContents, PageZeroState};

pub struct StreamUpdate<'a> {
    pub stream_id: u32,
    pub data: &'a [u8],
}

pub struct IncrementalResult {
    pub bytes: Vec<u8>,
    pub new_contents: PageContents,
    pub stream_map: HashMap<u32, u32>,
}

pub fn incremental_update(
    base_bytes: &[u8],
    base_doc: &lopdf::Document,
    page: &PageZeroState,
    updates: &[StreamUpdate<'_>],
) -> Result<IncrementalResult> {
    if updates.is_empty() {
        return Err(anyhow!("no stream updates provided"));
    }

    let mut incremental = IncrementalDocument::create_from(base_bytes.to_vec(), base_doc.clone());
    let mut next_id = base_doc.max_id + 1;
    let mut mapping = HashMap::new();

    for update in updates {
        let new_id = next_id;
        next_id += 1;
        mapping.insert(update.stream_id, new_id);
        let mut dict = Dictionary::new();
        dict.set("Length", Object::Integer(update.data.len() as i64));
        let stream = Stream::new(dict, update.data.to_vec());
        incremental
            .new_document
            .objects
            .insert((new_id, 0), Object::Stream(stream));
    }

    if next_id > base_doc.max_id {
        incremental.new_document.max_id = next_id - 1;
    }
    let size = incremental.new_document.max_id + 1;
    incremental
        .new_document
        .trailer
        .set("Size", Object::Integer(size as i64));

    let page_id = (page.page_id, 0);
    incremental.opt_clone_object_to_new_document(page_id)?;
    let page_obj = incremental
        .new_document
        .get_object_mut(page_id)?
        .as_dict_mut()?;

    let updated_contents = match &page.contents {
        PageContents::Single(id) => {
            let new_id = mapping.get(id).copied().unwrap_or(*id);
            PageContents::Single(new_id)
        }
        PageContents::Array(ids) => {
            let mapped = ids
                .iter()
                .map(|id| mapping.get(id).copied().unwrap_or(*id))
                .collect();
            PageContents::Array(mapped)
        }
    };

    let contents_object = match &updated_contents {
        PageContents::Single(id) => Object::Reference((*id, 0)),
        PageContents::Array(ids) => {
            Object::Array(ids.iter().map(|id| Object::Reference((*id, 0))).collect())
        }
    };
    page_obj.set("Contents", contents_object);

    let mut buffer = Vec::new();
    incremental.save_to(&mut buffer)?;

    Ok(IncrementalResult {
        bytes: buffer,
        new_contents: updated_contents,
        stream_map: mapping,
    })
}
