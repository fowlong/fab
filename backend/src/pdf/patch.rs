//! Apply transform patches to PDF content streams.

use std::collections::HashMap;

use anyhow::{anyhow, Result};
use lopdf::{Document, ObjectId};

use crate::pdf::extract::{extract_ir, page_streams};
use crate::pdf::write;
use crate::types::{DocumentIR, ImageObject, PageObject, PatchKind, PatchOperation, TextObject};

pub struct PatchOutcome {
    pub bytes: Vec<u8>,
    pub document: Document,
    pub ir: DocumentIR,
}

pub fn apply_patches(
    doc: &Document,
    original_bytes: &[u8],
    ir: &DocumentIR,
    ops: &[PatchOperation],
) -> Result<PatchOutcome> {
    if ops.is_empty() {
        return Ok(PatchOutcome {
            bytes: original_bytes.to_vec(),
            document: doc.clone(),
            ir: ir.clone(),
        });
    }

    let (page_index, page_id) = doc
        .get_pages()
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("document has no pages"))?;
    if page_index != 1 {
        return Err(anyhow!("only the first page is supported in this stage"));
    }

    let mut text_changes: HashMap<String, TextChange> = HashMap::new();
    let mut image_changes: HashMap<String, ImageChange> = HashMap::new();

    for op in ops {
        if let PatchOperation::Transform {
            target,
            delta_matrix_pt,
            kind,
        } = op
        {
            if target.page != 0 {
                return Err(anyhow!("only page 0 edits are supported"));
            }
            match kind {
                PatchKind::Text => {
                    let text_obj = find_text(ir, &target.id)?;
                    let entry =
                        text_changes
                            .entry(target.id.clone())
                            .or_insert_with(|| TextChange {
                                object: text_obj.clone(),
                                matrix: text_obj.tm,
                            });
                    entry.matrix = multiply(*delta_matrix_pt, entry.matrix);
                }
                PatchKind::Image => {
                    let image_obj = find_image(ir, &target.id)?;
                    let entry =
                        image_changes
                            .entry(target.id.clone())
                            .or_insert_with(|| ImageChange {
                                object: image_obj.clone(),
                                matrix: image_obj.cm,
                            });
                    entry.matrix = multiply(*delta_matrix_pt, entry.matrix);
                }
            }
        }
    }

    let streams = page_streams(doc, page_id)?;
    let mut stream_map: HashMap<u32, Vec<u8>> = streams
        .iter()
        .map(|stream| (stream.object_id.0, stream.data.clone()))
        .collect();
    let order: Vec<u32> = streams.iter().map(|stream| stream.object_id.0).collect();

    for change in text_changes.values() {
        let stream_id = change.object.spans.block.stream_obj;
        let bytes = stream_map
            .get_mut(&stream_id)
            .ok_or_else(|| anyhow!("content stream {} not found", stream_id))?;
        let mut modifications = Vec::new();
        let formatted = format_matrix(change.matrix, "Tm");
        if let Some(tm_span) = &change.object.spans.tm_op {
            modifications.push(Modification {
                start: tm_span.start as usize,
                end: tm_span.end as usize,
                replacement: formatted.clone(),
            });
        } else {
            let insert_pos = change.object.spans.bt_op.end as usize;
            let insertion = format!("\n{}\n", formatted);
            modifications.push(Modification {
                start: insert_pos,
                end: insert_pos,
                replacement: insertion,
            });
        }
        apply_modifications(bytes, &mut modifications);
    }

    for change in image_changes.values() {
        let stream_id = change.object.span.stream_obj;
        let bytes = stream_map
            .get_mut(&stream_id)
            .ok_or_else(|| anyhow!("content stream {} not found", stream_id))?;
        let formatted = format_matrix(change.matrix, "cm");
        let mut modifications = vec![Modification {
            start: change.object.span.start as usize,
            end: change.object.span.end as usize,
            replacement: formatted,
        }];
        apply_modifications(bytes, &mut modifications);
    }

    let mut combined = Vec::new();
    for (index, stream_id) in order.iter().enumerate() {
        if index > 0 {
            combined.push(b'\n');
        }
        if let Some(bytes) = stream_map.get(stream_id) {
            combined.extend_from_slice(bytes);
        }
    }

    let (updated_bytes, updated_doc, _) =
        write::incremental_update(doc, original_bytes, page_id, combined)?;
    let updated_ir = extract_ir(&updated_doc)?;

    Ok(PatchOutcome {
        bytes: updated_bytes,
        document: updated_doc,
        ir: updated_ir,
    })
}

fn find_text<'a>(ir: &'a DocumentIR, id: &str) -> Result<&'a TextObject> {
    ir.pages
        .get(0)
        .ok_or_else(|| anyhow!("page 0 missing from IR"))?
        .objects
        .iter()
        .find_map(|obj| match obj {
            PageObject::Text(text) if text.id == id => Some(text),
            _ => None,
        })
        .ok_or_else(|| anyhow!("text object {id} not found"))
}

fn find_image<'a>(ir: &'a DocumentIR, id: &str) -> Result<&'a ImageObject> {
    ir.pages
        .get(0)
        .ok_or_else(|| anyhow!("page 0 missing from IR"))?
        .objects
        .iter()
        .find_map(|obj| match obj {
            PageObject::Image(image) if image.id == id => Some(image),
            _ => None,
        })
        .ok_or_else(|| anyhow!("image object {id} not found"))
}

struct TextChange {
    object: TextObject,
    matrix: [f64; 6],
}

struct ImageChange {
    object: ImageObject,
    matrix: [f64; 6],
}

struct Modification {
    start: usize,
    end: usize,
    replacement: String,
}

fn apply_modifications(bytes: &mut Vec<u8>, modifications: &mut [Modification]) {
    modifications.sort_by_key(|m| m.start);
    let mut shift: isize = 0;
    for modification in modifications.iter() {
        let start = (modification.start as isize + shift) as usize;
        let end = (modification.end as isize + shift) as usize;
        let replacement_bytes = modification.replacement.as_bytes();
        bytes.splice(start..end, replacement_bytes.iter().copied());
        shift += replacement_bytes.len() as isize - (end - start) as isize;
    }
}

fn multiply(delta: [f64; 6], base: [f64; 6]) -> [f64; 6] {
    [
        delta[0] * base[0] + delta[2] * base[1],
        delta[1] * base[0] + delta[3] * base[1],
        delta[0] * base[2] + delta[2] * base[3],
        delta[1] * base[2] + delta[3] * base[3],
        delta[0] * base[4] + delta[2] * base[5] + delta[4],
        delta[1] * base[4] + delta[3] * base[5] + delta[5],
    ]
}

fn format_matrix(matrix: [f64; 6], operator: &str) -> String {
    let parts: Vec<String> = matrix.iter().map(|value| format_number(*value)).collect();
    format!("{} {}", parts.join(" "), operator)
}

fn format_number(value: f64) -> String {
    let mut text = if (value - value.round()).abs() < 1e-6 {
        format!("{:.0}", value.round())
    } else {
        format!("{:.6}", value)
    };
    while text.contains('.') && text.ends_with('0') {
        text.pop();
    }
    if text.ends_with('.') {
        text.pop();
    }
    if text == "-0" {
        text = "0".into();
    }
    text
}
