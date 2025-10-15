use lopdf::{Dictionary, Document, ObjectId};

use super::Result;
use crate::types::{DocumentIr, PageIr};

pub fn extract_document_ir(bytes: &[u8]) -> Result<DocumentIr> {
    let document = Document::load_mem(bytes)?;
    let mut pages = Vec::new();
    for (index, (_page_number, object_id)) in document.get_pages().iter().enumerate() {
        let object_id = ObjectId::new(object_id.0, object_id.1);
        let page_dict = document
            .get_object(object_id)?
            .clone()
            .as_dict()?
            .to_owned();
        let media_box = extract_media_box(&page_dict).unwrap_or([0.0, 0.0, 595.0, 842.0]);
        pages.push(PageIr {
            index,
            width_pt: media_box[2] - media_box[0],
            height_pt: media_box[3] - media_box[1],
            objects: Vec::new(),
        });
    }
    Ok(DocumentIr { pages })
}

fn extract_media_box(dict: &Dictionary) -> Option<[f64; 4]> {
    dict.get(b"MediaBox")
        .and_then(|obj| obj.as_array())
        .map(|arr| {
            let mut values = [0.0; 4];
            for (idx, value) in arr.iter().take(4).enumerate() {
                values[idx] = value.as_f64().unwrap_or(0.0);
            }
            values
        })
}
