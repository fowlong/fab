use anyhow::{Context, Result};
use lopdf::{Document, Object};

use crate::types::{DocumentIR, PageIR};

pub fn extract_ir(bytes: &[u8]) -> Result<DocumentIR> {
    let doc = Document::load_mem(bytes).context("failed to parse PDF document")?;
    let pages_tree = doc.get_pages();
    let mut pages = Vec::new();
    for (index, (_, page_dict)) in pages_tree.into_iter().enumerate() {
        let (width_pt, height_pt) = media_box_dimensions(&page_dict).unwrap_or((595.0, 842.0));
        pages.push(PageIR {
            index,
            width_pt,
            height_pt,
            objects: Vec::new(),
        });
    }
    Ok(DocumentIR { pages })
}

fn media_box_dimensions(dict: &lopdf::Dictionary) -> Option<(f64, f64)> {
    let array = dict.get(b"MediaBox").ok()?;
    let values = match array {
        Object::Array(values) if values.len() == 4 => values,
        _ => return None,
    };
    let x0 = values[0].as_f64().ok()?;
    let y0 = values[1].as_f64().ok()?;
    let x1 = values[2].as_f64().ok()?;
    let y1 = values[3].as_f64().ok()?;
    Some((x1 - x0, y1 - y0))
}
