//! Apply transform patches to PDF content streams.

use std::collections::HashMap;

use anyhow::{anyhow, Result};
use lopdf::{Dictionary, Object, Stream};

use crate::types::{PatchOperation, TransformKind};

use super::{
    content::{self, format_matrix, multiply, Operator},
    extract::{operands_to_matrix, PageExtraction, StreamContent},
    loader::LoadedDoc,
    write,
};

pub fn apply_patches(
    doc: &mut LoadedDoc,
    extraction: &PageExtraction,
    ops: &[PatchOperation],
) -> Result<()> {
    if ops.is_empty() {
        return Ok(());
    }

    let mut streams = extraction.streams.clone();
    let mut text_objects = collect_text_objects(&extraction.ir);
    let mut image_objects = collect_image_objects(&extraction.ir);

    for op in ops {
        match op {
            PatchOperation::Transform {
                target,
                delta_matrix_pt,
                kind,
            } => {
                if target.page != 0 {
                    continue;
                }
                match kind {
                    TransformKind::Text => {
                        let text = text_objects
                            .get_mut(&target.id)
                            .ok_or_else(|| anyhow!("unknown text target {}", target.id))?;
                        let stream_id = text.bt_span.stream_obj;
                        let stream = find_stream_mut(&mut streams, stream_id)?;
                        let new_tm = multiply(*delta_matrix_pt, text.tm);
                        patch_text(stream, text, new_tm)?;
                        text.tm = new_tm;
                    }
                    TransformKind::Image => {
                        let image = image_objects
                            .get_mut(&target.id)
                            .ok_or_else(|| anyhow!("unknown image target {}", target.id))?;
                        let stream_id = find_image_stream_id(extraction, &target.id)?;
                        let stream = find_stream_mut(&mut streams, stream_id)?;
                        patch_image(stream, extraction, &target.id, *delta_matrix_pt, image)?;
                        image.cm = multiply(*delta_matrix_pt, image.cm);
                    }
                }
            }
        }
    }

    let combined = combine_streams(&streams);

    let prev_doc = doc.doc.clone();
    let mut updated_doc = doc.doc.clone();

    let stream_object = Stream::new(Dictionary::new(), combined.clone());
    let new_stream_id = updated_doc.add_object(stream_object.clone());
    {
        let page_dict = updated_doc
            .get_object_mut(extraction.page_id)
            .and_then(Object::as_dict_mut)?;
        page_dict.set("Contents", Object::Reference(new_stream_id));
    }

    let page_object = updated_doc.get_object(extraction.page_id)?.clone();

    let updates = vec![
        (new_stream_id, Object::Stream(stream_object)),
        (extraction.page_id, page_object.clone()),
    ];

    let bytes = write::incremental_update(&prev_doc, &doc.bytes, &updates)?;

    doc.doc = updated_doc;
    doc.bytes = bytes;
    doc.invalidate_page0();

    Ok(())
}

fn collect_text_objects(page: &crate::types::PageIR) -> HashMap<String, crate::types::TextObject> {
    page.objects
        .iter()
        .filter_map(|obj| match obj {
            crate::types::PageObject::Text(text) => Some((text.id.clone(), text.clone())),
            _ => None,
        })
        .collect()
}

fn collect_image_objects(
    page: &crate::types::PageIR,
) -> HashMap<String, crate::types::ImageObject> {
    page.objects
        .iter()
        .filter_map(|obj| match obj {
            crate::types::PageObject::Image(image) => Some((image.id.clone(), image.clone())),
            _ => None,
        })
        .collect()
}

fn find_stream_mut<'a>(streams: &'a mut [StreamContent], id: u32) -> Result<&'a mut StreamContent> {
    streams
        .iter_mut()
        .find(|stream| stream.object_id == id)
        .ok_or_else(|| anyhow!("missing content stream {id}"))
}

fn find_image_stream_id(extraction: &PageExtraction, id: &str) -> Result<u32> {
    extraction
        .image_lookup
        .get(id)
        .map(|info| info.stream_obj)
        .ok_or_else(|| anyhow!("missing image lookup for {id}"))
}

fn patch_text(
    stream: &mut StreamContent,
    text: &crate::types::TextObject,
    new_tm: [f64; 6],
) -> Result<()> {
    let bt_start = text.bt_span.start as usize;
    let tokens = &stream.tokens;
    let bt_index = tokens
        .iter()
        .position(|token| token.operator == Operator::BT && token.span.start == bt_start)
        .ok_or_else(|| anyhow!("BT token not found for text {}", text.id))?;

    let tm_index = tokens[bt_index + 1..]
        .iter()
        .position(|token| token.operator == Operator::Tm)
        .map(|idx| idx + bt_index + 1);

    if let Some(idx) = tm_index {
        let span = stream.tokens[idx].span.clone();
        let replacement = format!("{} Tm", format_matrix(new_tm));
        stream.bytes = replace_range(&stream.bytes, span, replacement.as_bytes());
    } else {
        let insert_pos = stream.tokens[bt_index].span.end;
        let insertion = format!("{} Tm\n", format_matrix(new_tm));
        stream.bytes = insert_bytes(&stream.bytes, insert_pos, insertion.as_bytes());
    }

    stream.tokens = content::tokenize_stream(&stream.bytes)?;
    Ok(())
}

fn patch_image(
    stream: &mut StreamContent,
    extraction: &PageExtraction,
    id: &str,
    delta: [f64; 6],
    image: &crate::types::ImageObject,
) -> Result<()> {
    let lookup = extraction
        .image_lookup
        .get(id)
        .ok_or_else(|| anyhow!("missing image lookup for {id}"))?;
    let span = lookup
        .cm_span
        .clone()
        .ok_or_else(|| anyhow!("image {id} has no cm operator to update"))?;

    let cm_index = stream
        .tokens
        .iter()
        .position(|token| token.operator == Operator::Cm && token.span.start == span.start)
        .ok_or_else(|| anyhow!("cm token not found for image {id}"))?;

    let existing = operands_to_matrix(&stream.tokens[cm_index].operands).unwrap_or(image.cm);
    let new_matrix = multiply(delta, existing);
    let replacement = format!("{} cm", format_matrix(new_matrix));
    stream.bytes = replace_range(
        &stream.bytes,
        stream.tokens[cm_index].span.clone(),
        replacement.as_bytes(),
    );
    stream.tokens = content::tokenize_stream(&stream.bytes)?;
    Ok(())
}

fn combine_streams(streams: &[StreamContent]) -> Vec<u8> {
    let mut combined = Vec::new();
    for (idx, stream) in streams.iter().enumerate() {
        if idx > 0 && !combined.ends_with(b"\n") {
            combined.push(b'\n');
        }
        combined.extend_from_slice(&stream.bytes);
    }
    combined
}

fn replace_range(bytes: &[u8], range: std::ops::Range<usize>, replacement: &[u8]) -> Vec<u8> {
    let mut output =
        Vec::with_capacity(bytes.len() - (range.end - range.start) + replacement.len());
    output.extend_from_slice(&bytes[..range.start]);
    output.extend_from_slice(replacement);
    output.extend_from_slice(&bytes[range.end..]);
    output
}

fn insert_bytes(bytes: &[u8], index: usize, addition: &[u8]) -> Vec<u8> {
    let mut output = Vec::with_capacity(bytes.len() + addition.len());
    output.extend_from_slice(&bytes[..index]);
    output.extend_from_slice(addition);
    output.extend_from_slice(&bytes[index..]);
    output
}
