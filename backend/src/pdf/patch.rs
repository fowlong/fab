//! Transform patch application for text runs and image XObjects.

use anyhow::{anyhow, Result};
use lopdf::{Document, Object, ObjectId};

use crate::types::{PatchOperation, TransformKind};

use super::extract::{ImageMeta, ObjectMeta, PageCache, TextMeta};
use super::loader;

#[derive(Debug)]
pub struct PatchPlan {
    pub updates: Vec<(ObjectId, Object)>,
    pub next_object_id: u32,
}

pub fn apply_transform(
    document: &mut Document,
    cache: &PageCache,
    op: &PatchOperation,
    next_object_id: u32,
) -> Result<PatchPlan> {
    match op {
        PatchOperation::Transform {
            target,
            delta_matrix_pt,
            kind,
        } => {
            if target.page != 0 {
                return Err(anyhow!("Only page 0 is supported in this stage"));
            }
            let meta = cache
                .objects
                .get(&target.id)
                .ok_or_else(|| anyhow!("Target not found in cache"))?;
            match (kind, meta) {
                (TransformKind::Text, ObjectMeta::Text(text_meta)) => {
                    apply_text_transform(document, cache, text_meta, delta_matrix_pt, next_object_id)
                }
                (TransformKind::Image, ObjectMeta::Image(image_meta)) => {
                    apply_image_transform(document, cache, image_meta, delta_matrix_pt, next_object_id)
                }
                _ => Err(anyhow!("Mismatched transform kind")),
            }
        }
    }
}

fn apply_text_transform(
    document: &mut Document,
    cache: &PageCache,
    meta: &TextMeta,
    delta: &[f64; 6],
    next_object_id: u32,
) -> Result<PatchPlan> {
    let stream_id: ObjectId = (meta.stream_obj, 0).into();
    let original = loader::load_stream_bytes(document, stream_id)?;
    let updated = if let Some(span) = &meta.tm_span {
        replace_span(
            &original,
            span.start as usize,
            span.end as usize,
            &format_matrix(&multiply(delta, &meta.tm), "Tm"),
        )
    } else {
        insert_span(
            &original,
            meta.insert_after_bt as usize,
            &format_matrix(&multiply(delta, &meta.tm), "Tm"),
        )
    };
    let new_stream_id: ObjectId = (next_object_id, 0).into();
    let stream_object = Object::Stream(loader::create_stream(updated));
    let mut page_dict = document
        .get_object(cache.page_id)?
        .as_dict()
        .map_err(|_| anyhow!("Page dictionary missing"))?
        .clone();
    page_dict.set(
        b"Contents",
        Object::Reference(new_stream_id),
    );
    let page_object = Object::Dictionary(page_dict);
    document.objects.insert(new_stream_id, stream_object.clone());
    document.objects.insert(cache.page_id, page_object.clone());
    Ok(PatchPlan {
        updates: vec![(new_stream_id, stream_object), (cache.page_id, page_object)],
        next_object_id: next_object_id + 1,
    })
}

fn apply_image_transform(
    document: &mut Document,
    cache: &PageCache,
    meta: &ImageMeta,
    delta: &[f64; 6],
    next_object_id: u32,
) -> Result<PatchPlan> {
    let stream_id: ObjectId = (meta.stream_obj, 0).into();
    let original = loader::load_stream_bytes(document, stream_id)?;
    let replacement = format_matrix(&multiply(delta, &meta.cm), "cm");
    let updated = replace_span(
        &original,
        meta.cm_span.start as usize,
        meta.cm_span.end as usize,
        &replacement,
    );
    let new_stream_id: ObjectId = (next_object_id, 0).into();
    let stream_object = Object::Stream(loader::create_stream(updated));
    let mut page_dict = document
        .get_object(cache.page_id)?
        .as_dict()
        .map_err(|_| anyhow!("Page dictionary missing"))?
        .clone();
    page_dict.set(
        b"Contents",
        Object::Reference(new_stream_id),
    );
    let page_object = Object::Dictionary(page_dict);
    document.objects.insert(new_stream_id, stream_object.clone());
    document.objects.insert(cache.page_id, page_object.clone());
    Ok(PatchPlan {
        updates: vec![(new_stream_id, stream_object), (cache.page_id, page_object)],
        next_object_id: next_object_id + 1,
    })
}

fn replace_span(original: &[u8], start: usize, end: usize, replacement: &str) -> Vec<u8> {
    let mut output = Vec::with_capacity(original.len() + replacement.len());
    output.extend_from_slice(&original[..start]);
    output.extend_from_slice(replacement.as_bytes());
    output.extend_from_slice(&original[end..]);
    output
}

fn insert_span(original: &[u8], offset: usize, replacement: &str) -> Vec<u8> {
    let mut output = Vec::with_capacity(original.len() + replacement.len());
    output.extend_from_slice(&original[..offset]);
    if !replacement.is_empty() {
        let mut snippet = String::new();
        snippet.push('\n');
        snippet.push_str(replacement);
        if !replacement.ends_with('\n') {
            snippet.push('\n');
        }
        output.extend_from_slice(snippet.as_bytes());
    }
    output.extend_from_slice(&original[offset..]);
    output
}

fn format_matrix(matrix: &[f64; 6], operator: &str) -> String {
    format!(
        "{:.6} {:.6} {:.6} {:.6} {:.6} {:.6} {operator}",
        matrix[0], matrix[1], matrix[2], matrix[3], matrix[4], matrix[5]
    )
}

fn multiply(a: &[f64; 6], b: &[f64; 6]) -> [f64; 6] {
    [
        a[0] * b[0] + a[2] * b[1],
        a[1] * b[0] + a[3] * b[1],
        a[0] * b[2] + a[2] * b[3],
        a[1] * b[2] + a[3] * b[3],
        a[0] * b[4] + a[2] * b[5] + a[4],
        a[1] * b[4] + a[3] * b[5] + a[5],
    ]
}
