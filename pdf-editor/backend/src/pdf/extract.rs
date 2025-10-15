use anyhow::{Context, Result};
use lopdf::{Document, Object};

use crate::{
    pdf::loader::StoredDocument,
    types::{IrDocument, IrObject, IrPage},
};

pub fn extract_ir(doc: &StoredDocument) -> Result<IrDocument> {
    let pdf = Document::load_mem(doc.latest_bytes())
        .with_context(|| "failed to parse PDF bytes for IR extraction")?;

    let mut pages: Vec<IrPage> = Vec::new();
    let mut page_entries: Vec<(u32, lopdf::ObjectId)> = pdf.get_pages().into_iter().collect();
    page_entries.sort_by_key(|(number, _)| *number);

    for (position, (_, object_id)) in page_entries.iter().enumerate() {
        let dictionary = pdf
            .get_dictionary(object_id)
            .with_context(|| format!("missing page dictionary for object {object_id:?}"))?;

        let (width, height) = parse_media_box(dictionary)?;

        pages.push(IrPage {
            index: position as u32,
            width_pt: width,
            height_pt: height,
            objects: Vec::new(),
        });
    }

    Ok(IrDocument { pages })
}

fn parse_media_box(dict: &lopdf::Dictionary) -> Result<(f64, f64)> {
    let media_box = dict
        .get(b"MediaBox")
        .or_else(|_| dict.get(b"CropBox"))
        .cloned()
        .unwrap_or_else(|| Object::Array(vec![0.into(), 0.into(), 612.into(), 792.into()]));

    match media_box {
        Object::Array(values) if values.len() == 4 => {
            let x1 = number_to_f64(&values[0])?;
            let y1 = number_to_f64(&values[1])?;
            let x2 = number_to_f64(&values[2])?;
            let y2 = number_to_f64(&values[3])?;
            Ok((x2 - x1, y2 - y1))
        }
        _ => Ok((612.0, 792.0)),
    }
}

fn number_to_f64(obj: &Object) -> Result<f64> {
    match obj {
        Object::Integer(i) => Ok(*i as f64),
        Object::Real(r) => Ok(*r as f64),
        _ => anyhow::bail!("unexpected number type in MediaBox"),
    }
}

#[allow(dead_code)]
pub fn flatten_objects(_objects: &[IrObject]) -> Vec<IrObject> {
    // Placeholder for future IR transformations.
    Vec::new()
}
