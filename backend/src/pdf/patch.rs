//! Apply transformation patches to PDF content streams.

use anyhow::{bail, Context, Result};
use lopdf::Document;

use crate::pdf::extract::ExtractedDocument;
use crate::pdf::write::{incremental_update, StreamUpdate};
use crate::types::{PatchOperation, TransformKind};

#[derive(Debug, Clone)]
pub struct PatchApply {
    pub bytes: Vec<u8>,
    pub document: Document,
    pub extraction: ExtractedDocument,
}

pub fn apply_patch(
    document: &Document,
    bytes: &[u8],
    extraction: &ExtractedDocument,
    op: &PatchOperation,
) -> Result<PatchApply> {
    match op {
        PatchOperation::Transform {
            target,
            delta_matrix_pt,
            kind,
        } => {
            if target.page != 0 {
                bail!("only page 0 is supported in this stage");
            }
            let page = extraction
                .page0
                .as_ref()
                .context("page 0 metadata missing")?;
            match kind {
                TransformKind::Text => {
                    apply_text_transform(document, bytes, page, target.id.as_str(), delta_matrix_pt)
                }
                TransformKind::Image => apply_image_transform(
                    document,
                    bytes,
                    page,
                    target.id.as_str(),
                    delta_matrix_pt,
                ),
            }
        }
    }
}

fn apply_text_transform(
    document: &Document,
    bytes: &[u8],
    page: &crate::pdf::extract::ExtractedPage,
    target_id: &str,
    delta: &[f64; 6],
) -> Result<PatchApply> {
    let meta = page
        .meta
        .text_map
        .get(target_id)
        .with_context(|| format!("text object {target_id} not found"))?;
    let stream = page
        .meta
        .streams
        .get(&meta.stream_obj.0)
        .with_context(|| format!("content stream {} missing", meta.stream_obj.0))?;
    let mut updated = stream.data.clone();
    let new_tm = multiply(*delta, meta.base_tm);
    let replacement = format_matrix(new_tm, "Tm");
    let replacement_bytes = format!("{replacement}\n").into_bytes();

    if let Some((start, end)) = meta.tm_span {
        splice(&mut updated, start, end, &replacement_bytes);
    } else if let Some((start, end)) = meta
        .translation_spans
        .first()
        .zip(meta.translation_spans.last())
        .map(|(first, last)| (first.0, last.1))
    {
        splice(&mut updated, start, end, &replacement_bytes);
    } else {
        let insert_at = meta.bt_token_span.1;
        splice(&mut updated, insert_at, insert_at, &replacement_bytes);
    }

    let (bytes, document) = incremental_update(
        bytes,
        document,
        &[StreamUpdate {
            object_id: stream.id,
            content: updated,
        }],
    )?;
    let extraction = crate::pdf::extract::extract_document(&document)?;
    Ok(PatchApply {
        bytes,
        document,
        extraction,
    })
}

fn apply_image_transform(
    document: &Document,
    bytes: &[u8],
    page: &crate::pdf::extract::ExtractedPage,
    target_id: &str,
    delta: &[f64; 6],
) -> Result<PatchApply> {
    let meta = page
        .meta
        .image_map
        .get(target_id)
        .with_context(|| format!("image object {target_id} not found"))?;
    let stream = page
        .meta
        .streams
        .get(&meta.stream_obj.0)
        .with_context(|| format!("content stream {} missing", meta.stream_obj.0))?;
    let span = meta
        .cm_span
        .context("image transform must reference an explicit cm operator")?;
    let mut updated = stream.data.clone();
    let new_cm = multiply(*delta, meta.cm_matrix);
    let replacement_bytes = format!("{}\n", format_matrix(new_cm, "cm")).into_bytes();
    splice(&mut updated, span.0, span.1, &replacement_bytes);

    let (bytes, document) = incremental_update(
        bytes,
        document,
        &[StreamUpdate {
            object_id: stream.id,
            content: updated,
        }],
    )?;
    let extraction = crate::pdf::extract::extract_document(&document)?;
    Ok(PatchApply {
        bytes,
        document,
        extraction,
    })
}

fn splice(buffer: &mut Vec<u8>, start: usize, end: usize, replacement: &[u8]) {
    if start > end || end > buffer.len() {
        return;
    }
    buffer.splice(start..end, replacement.iter().copied());
}

fn format_matrix(matrix: [f64; 6], operator: &str) -> String {
    format!(
        "{:.6} {:.6} {:.6} {:.6} {:.6} {:.6} {}",
        matrix[0], matrix[1], matrix[2], matrix[3], matrix[4], matrix[5], operator
    )
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
