use anyhow::Context;
use lopdf::{Document, Object};

use crate::types::{DocumentIR, PageIR};

const DEFAULT_WIDTH: f64 = 595.276;
const DEFAULT_HEIGHT: f64 = 841.89;

pub fn extract_ir(doc: &Document) -> anyhow::Result<DocumentIR> {
    let mut pages = Vec::new();

    for (index, page_id) in doc.page_iter().enumerate() {
        let page_obj = doc
            .get_object(page_id)
            .with_context(|| format!("missing page object for id {page_id:?}"))?;
        let width_height = page_dimensions(page_obj).unwrap_or((DEFAULT_WIDTH, DEFAULT_HEIGHT));
        pages.push(PageIR {
            index,
            width_pt: width_height.0,
            height_pt: width_height.1,
            objects: Vec::new(),
        });
    }

    Ok(DocumentIR { pages })
}

fn page_dimensions(object: &Object) -> Option<(f64, f64)> {
    let dict = object.as_dict().ok()?;
    let media_box = dict.get(b"MediaBox")?;
    match media_box {
        Object::Array(values) if values.len() == 4 => {
            let x0 = values[0].as_f64().ok().unwrap_or(0.0);
            let y0 = values[1].as_f64().ok().unwrap_or(0.0);
            let x1 = values[2].as_f64().ok().unwrap_or(DEFAULT_WIDTH);
            let y1 = values[3].as_f64().ok().unwrap_or(DEFAULT_HEIGHT);
            Some((x1 - x0, y1 - y0))
        }
        _ => None,
    }
}
