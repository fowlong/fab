use anyhow::{anyhow, Context, Result};
use lopdf::{Document, Object, ObjectId};

use crate::types::{DocumentIr, PageIr};

pub fn extract_ir(bytes: &[u8]) -> Result<DocumentIr> {
    let document = Document::load_mem(bytes)?;
    let mut pages = Vec::new();

    for (index, page_id) in document.page_iter().enumerate() {
        let size = page_size(&document, page_id)?;
        pages.push(PageIr {
            index,
            width_pt: size.0,
            height_pt: size.1,
            objects: Vec::new(),
        });
    }

    Ok(DocumentIr { pages })
}

fn page_size(document: &Document, page_id: ObjectId) -> Result<(f64, f64)> {
    let page_dict = document.get_dictionary(page_id)?;
    let media_box = page_dict
        .get(b"MediaBox")
        .or_else(|| page_dict.get(b"CropBox"))
        .context("page missing MediaBox/CropBox")?;
    let (llx, lly, urx, ury) = match media_box {
        Object::Array(items) if items.len() == 4 => {
            let llx = to_f64(&items[0])?;
            let lly = to_f64(&items[1])?;
            let urx = to_f64(&items[2])?;
            let ury = to_f64(&items[3])?;
            (llx, lly, urx, ury)
        }
        _ => return Err(anyhow!("invalid MediaBox")),
    };
    Ok((urx - llx, ury - lly))
}

fn to_f64(object: &Object) -> Result<f64> {
    match object {
        Object::Integer(value) => Ok(*value as f64),
        Object::Real(value) => Ok(*value as f64),
        _ => Err(anyhow!("unexpected numeric object")),
    }
}
