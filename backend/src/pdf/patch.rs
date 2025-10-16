//! Patch application for transform operations.

use std::fs;

use anyhow::{anyhow, Context, Result};

use crate::types::{PageObject, PatchOperation, TransformKind};

use super::{
    extract::{extract_page0, ImageMeta, ObjectMeta, Page0Cache, TextMeta},
    loader::LoadedDoc,
    write,
};

pub fn apply_patches(doc: &mut LoadedDoc, ops: &[PatchOperation]) -> Result<()> {
    if ops.is_empty() {
        return Ok(());
    }
    ensure_cache(doc)?;
    for op in ops {
        match op {
            PatchOperation::Transform {
                target,
                delta_matrix_pt,
                kind,
            } => {
                if target.page != 0 {
                    return Err(anyhow!("only page 0 is supported in the MVP"));
                }
                match kind {
                    TransformKind::Text => apply_text_transform(doc, &target.id, delta_matrix_pt)?,
                    TransformKind::Image => {
                        apply_image_transform(doc, &target.id, delta_matrix_pt)?
                    }
                }
            }
        }
        let cache = extract_page0(&doc.doc)?;
        doc.page0_cache = Some(cache);
    }
    Ok(())
}

fn ensure_cache(doc: &mut LoadedDoc) -> Result<()> {
    if doc.page0_cache.is_none() {
        let cache = extract_page0(&doc.doc)?;
        doc.page0_cache = Some(cache);
    }
    Ok(())
}
fn apply_text_transform(doc: &mut LoadedDoc, id: &str, delta: &[f64; 6]) -> Result<()> {
    let cache = doc
        .page0_cache
        .as_ref()
        .ok_or_else(|| anyhow!("missing IR cache"))?;
    let (text_obj, meta) = find_text(cache, id)?;
    let stream_cache = cache
        .streams
        .get(&meta.stream_obj)
        .ok_or_else(|| anyhow!("stream {} missing", meta.stream_obj))?;
    let new_tm = multiply(*delta, text_obj.tm);
    let mut updated_stream = stream_cache.bytes.clone();

    if let Some(range) = &meta.tm_range {
        replace_matrix(&mut updated_stream, range.clone(), new_tm, "Tm")?;
    } else {
        let insertion = format!(" {} Tm\n", format_matrix(new_tm));
        updated_stream.splice(
            meta.insert_after_bt..meta.insert_after_bt,
            insertion.into_bytes(),
        );
    }

    write_update(doc, cache.page_id, updated_stream)?
}

fn apply_image_transform(doc: &mut LoadedDoc, id: &str, delta: &[f64; 6]) -> Result<()> {
    let cache = doc
        .page0_cache
        .as_ref()
        .ok_or_else(|| anyhow!("missing IR cache"))?;
    let (image_obj, meta) = find_image(cache, id)?;
    let stream_cache = cache
        .streams
        .get(&meta.stream_obj)
        .ok_or_else(|| anyhow!("stream {} missing", meta.stream_obj))?;
    let new_cm = multiply(*delta, image_obj.cm);
    let mut updated_stream = stream_cache.bytes.clone();
    replace_matrix(&mut updated_stream, meta.cm_range.clone(), new_cm, "cm")?;
    write_update(doc, cache.page_id, updated_stream)?
}
fn find_text<'a>(
    cache: &'a Page0Cache,
    id: &str,
) -> Result<(&'a crate::types::TextObject, TextMeta)> {
    let objects = &cache.ir.pages[0].objects;
    let object_map = &cache.objects;
    let meta = match object_map.get(id) {
        Some(ObjectMeta::Text(meta)) => meta.clone(),
        _ => return Err(anyhow!("text object {id} not found")),
    };
    let text = objects
        .iter()
        .filter_map(|obj| match obj {
            PageObject::Text(text) if text.id == id => Some(text),
            _ => None,
        })
        .next()
        .ok_or_else(|| anyhow!("text object {id} not found in IR"))?;
    Ok((text, meta))
}

fn find_image<'a>(
    cache: &'a Page0Cache,
    id: &str,
) -> Result<(&'a crate::types::ImageObject, ImageMeta)> {
    let objects = &cache.ir.pages[0].objects;
    let object_map = &cache.objects;
    let meta = match object_map.get(id) {
        Some(ObjectMeta::Image(meta)) => meta.clone(),
        _ => return Err(anyhow!("image object {id} not found")),
    };
    let image = objects
        .iter()
        .filter_map(|obj| match obj {
            PageObject::Image(image) if image.id == id => Some(image),
            _ => None,
        })
        .next()
        .ok_or_else(|| anyhow!("image object {id} not found in IR"))?;
    Ok((image, meta))
}

fn replace_matrix(
    stream: &mut Vec<u8>,
    range: std::ops::Range<usize>,
    matrix: [f64; 6],
    operator: &str,
) -> Result<()> {
    if range.end > stream.len() || range.start > range.end {
        return Err(anyhow!("invalid token range"));
    }
    let slice = &stream[range.clone()];
    let trailing_newline = slice.ends_with(b"\n");
    let replacement = if trailing_newline {
        format!("{} {}\n", format_matrix(matrix), operator)
    } else {
        format!("{} {}", format_matrix(matrix), operator)
    };
    stream.splice(range, replacement.into_bytes());
    Ok(())
}

fn write_update(
    doc: &mut LoadedDoc,
    page_id: lopdf::ObjectId,
    stream_bytes: Vec<u8>,
) -> Result<()> {
    let prev_bytes = doc.bytes.clone();
    let previous_doc = std::mem::replace(&mut doc.doc, lopdf::Document::new());
    match write::incremental_update(prev_bytes, previous_doc, page_id, stream_bytes) {
        Ok((updated_bytes, updated_doc)) => {
            fs::write(&doc.path, &updated_bytes).with_context(|| {
                format!("failed to persist updated PDF to {}", doc.path.display())
            })?;
            doc.bytes = updated_bytes;
            doc.doc = updated_doc;
            Ok(())
        }
        Err(err) => {
            doc.doc = lopdf::Document::load_mem(&doc.bytes)?;
            Err(err)
        }
    }
}

fn format_matrix(matrix: [f64; 6]) -> String {
    matrix
        .iter()
        .map(|value| format!("{:.6}", value))
        .collect::<Vec<_>>()
        .join(" ")
}

fn multiply(a: [f64; 6], b: [f64; 6]) -> [f64; 6] {
    [
        a[0] * b[0] + a[2] * b[1],
        a[1] * b[0] + a[3] * b[1],
        a[0] * b[2] + a[2] * b[3],
        a[1] * b[2] + a[3] * b[3],
        a[0] * b[4] + a[2] * b[5] + a[4],
        a[1] * b[4] + a[3] * b[5] + a[5],
    ]
}
