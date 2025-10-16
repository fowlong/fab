//! Patch application for transform operations.

use anyhow::{anyhow, Result};
use lopdf::Document;

use crate::pdf::extract::{rebuild_page, ContentStream, ImageMeta, PageExtraction, TextRunMeta};
use crate::types::{PatchOperation, PatchTransformKind};

pub fn apply_patches(
    page: &mut PageExtraction,
    ops: &[PatchOperation],
    doc: &Document,
) -> Result<bool> {
    let mut modified = false;
    for op in ops {
        match op {
            PatchOperation::Transform {
                target,
                delta_matrix_pt,
                kind,
            } => {
                if target.page != 0 {
                    return Err(anyhow!("only page 0 is supported"));
                }
                match kind {
                    PatchTransformKind::Text => {
                        let meta = page
                            .text_map
                            .get(&target.id)
                            .cloned()
                            .ok_or_else(|| anyhow!("text target {} missing", target.id))?;
                        apply_text_transform(page, &meta, *delta_matrix_pt)?;
                        rebuild_page(page, doc)?;
                        modified = true;
                    }
                    PatchTransformKind::Image => {
                        let meta = page
                            .image_map
                            .get(&target.id)
                            .cloned()
                            .ok_or_else(|| anyhow!("image target {} missing", target.id))?;
                        apply_image_transform(page, &meta, *delta_matrix_pt)?;
                        rebuild_page(page, doc)?;
                        modified = true;
                    }
                }
            }
        }
    }
    Ok(modified)
}

pub fn combine_streams(streams: &[ContentStream]) -> Vec<u8> {
    let mut combined = Vec::new();
    for (index, stream) in streams.iter().enumerate() {
        if index > 0 {
            combined.push(b'\n');
        }
        combined.extend_from_slice(&stream.bytes);
    }
    combined
}

fn apply_text_transform(
    page: &mut PageExtraction,
    meta: &TextRunMeta,
    delta: [f64; 6],
) -> Result<()> {
    let stream = page
        .streams
        .get_mut(meta.stream_index)
        .ok_or_else(|| anyhow!("text stream index out of range"))?;

    let new_tm = multiply(delta, meta.tm);
    let replacement = format_matrix(new_tm, "Tm");

    if let Some(span) = meta.tm_span.clone() {
        replace_range(
            &mut stream.bytes,
            span.start,
            span.end,
            replacement.into_bytes(),
        );
    } else {
        let original = format_matrix(meta.tm, "Tm").into_bytes();
        let original_len = original.len();
        let insert_at = meta.insert_offset.min(stream.bytes.len());
        stream.bytes.splice(insert_at..insert_at, original);
        replace_range(
            &mut stream.bytes,
            insert_at,
            insert_at + original_len,
            replacement.into_bytes(),
        );
    }
    Ok(())
}

fn apply_image_transform(
    page: &mut PageExtraction,
    meta: &ImageMeta,
    delta: [f64; 6],
) -> Result<()> {
    let stream = page
        .streams
        .get_mut(meta.stream_index)
        .ok_or_else(|| anyhow!("image stream index out of range"))?;
    let new_cm = multiply(delta, meta.raw_cm);
    let replacement = format_matrix(new_cm, "cm");
    replace_range(
        &mut stream.bytes,
        meta.cm_span.start,
        meta.cm_span.end,
        replacement.into_bytes(),
    );
    Ok(())
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

fn format_matrix(matrix: [f64; 6], operator: &str) -> String {
    format!(
        "{:.6} {:.6} {:.6} {:.6} {:.6} {:.6} {}\n",
        matrix[0], matrix[1], matrix[2], matrix[3], matrix[4], matrix[5], operator
    )
}

fn replace_range(bytes: &mut Vec<u8>, start: usize, end: usize, replacement: Vec<u8>) {
    let end = end.min(bytes.len());
    bytes.splice(start..end, replacement);
}
