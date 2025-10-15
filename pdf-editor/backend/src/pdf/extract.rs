use anyhow::{Context, Result};
use lopdf::{Document, Object};

use crate::types::{DocumentIr, PageIr};

pub fn extract_ir(doc: &Document) -> Result<DocumentIr> {
    let mut pages = Vec::new();
    for (index, (_page_num, page_id)) in doc.get_pages().into_iter().enumerate() {
        let page_obj = doc.get_object(page_id)?;
        let page_dict = page_obj.as_dict().context("page is not a dictionary")?;
        let media_box = page_dict
            .get(b"MediaBox")
            .and_then(|obj| doc.resolve(obj).ok())
            .and_then(|obj| obj.as_array().cloned())
            .unwrap_or_else(|| {
                vec![
                    Object::Integer(0),
                    Object::Integer(0),
                    Object::Integer(595),
                    Object::Integer(842),
                ]
            });
        let width = number_at(&media_box, 2).unwrap_or(595.0);
        let height = number_at(&media_box, 3).unwrap_or(842.0);
        pages.push(PageIr {
            index: index as u32,
            width_pt: width as f32,
            height_pt: height as f32,
            objects: Vec::new(),
        });
    }
    Ok(DocumentIr { pages })
}

fn number_at(array: &[Object], idx: usize) -> Option<f64> {
    array.get(idx).and_then(|obj| match obj {
        Object::Integer(i) => Some(*i as f64),
        Object::Real(r) => Some(*r as f64),
        Object::Reference(id) => {
            log::warn!("MediaBox references object {id:?}, defaulting to 0");
            Some(0.0)
        }
        _ => None,
    })
}
