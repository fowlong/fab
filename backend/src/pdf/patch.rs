use std::collections::{HashMap, HashSet};

use anyhow::{bail, Context, Result};
use lopdf::{Document, Object, ObjectId, Stream};

use crate::pdf::extract::{CachedImage, CachedObject, CachedText, ExtractedDocument};
use crate::pdf::write::{incremental_update, IncrementalWrite, SerializedObject};
use crate::types::{PatchKind, PatchOperation};

pub fn apply_patches(
    original: &[u8],
    extraction: &ExtractedDocument,
    ops: &[PatchOperation],
) -> Result<Vec<u8>> {
    if ops.is_empty() {
        return Ok(original.to_vec());
    }

    let mut doc = Document::load_mem(original)?;
    let mut trailer = doc.trailer.clone();
    let prev_startxref = if doc.xref_start > 0 {
        doc.xref_start as u64
    } else {
        find_startxref(original)?
    };

    let page_cache = extraction.pages.get(&0).context("page 0 cache missing")?;

    let mut stream_data: HashMap<usize, Vec<u8>> = HashMap::new();
    let mut stream_offsets: HashMap<usize, isize> = HashMap::new();
    let mut touched_streams: HashSet<usize> = HashSet::new();

    for op in ops {
        if let PatchOperation::Transform {
            target,
            delta_matrix_pt,
            kind,
        } = op
        {
            if target.page != 0 {
                bail!("only page 0 supported in MVP");
            }
            let delta = *delta_matrix_pt;
            match (kind, page_cache.objects.get(&target.id)) {
                (PatchKind::Text, Some(CachedObject::Text(info))) => {
                    let data = stream_data.entry(info.stream_index).or_insert_with(|| {
                        page_cache.contents.streams[info.stream_index].data.clone()
                    });
                    touched_streams.insert(info.stream_index);
                    let offset = *stream_offsets.get(&info.stream_index).unwrap_or(&0);
                    let base_tm = info.base_tm;
                    let new_tm = matrix_multiply(delta, base_tm);
                    let replacement = matrix_token(new_tm, "Tm");
                    if let Some((start, end)) = info.tm_range {
                        let adj_start = (start as isize + offset) as usize;
                        let adj_end = (end as isize + offset) as usize;
                        let original_bytes = data[adj_start..adj_end].to_vec();
                        let mut token_bytes = replacement.into_bytes();
                        if !ends_with_whitespace(&original_bytes) {
                            token_bytes.push(b'\n');
                        }
                        splice_bytes(data, adj_start..adj_end, &token_bytes);
                        let delta_len = token_bytes.len() as isize - (adj_end - adj_start) as isize;
                        stream_offsets
                            .entry(info.stream_index)
                            .and_modify(|value| *value += delta_len)
                            .or_insert(delta_len);
                    } else {
                        let insert_idx = (info.bt_token_end as isize + offset) as usize;
                        let mut token = matrix_token(new_tm, "Tm").into_bytes();
                        token.push(b'\n');
                        splice_bytes(data, insert_idx..insert_idx, &token);
                        let delta_len = token.len() as isize;
                        stream_offsets
                            .entry(info.stream_index)
                            .and_modify(|value| *value += delta_len)
                            .or_insert(delta_len);
                    }
                }
                (PatchKind::Image, Some(CachedObject::Image(info))) => {
                    let data = stream_data.entry(info.stream_index).or_insert_with(|| {
                        page_cache.contents.streams[info.stream_index].data.clone()
                    });
                    touched_streams.insert(info.stream_index);
                    let offset = *stream_offsets.get(&info.stream_index).unwrap_or(&0);
                    let start = (info.cm_range.0 as isize + offset) as usize;
                    let end = (info.cm_range.1 as isize + offset) as usize;
                    let new_cm = matrix_multiply(delta, info.explicit_cm);
                    let original_bytes = data[start..end].to_vec();
                    let mut replacement = matrix_token(new_cm, "cm").into_bytes();
                    if !ends_with_whitespace(&original_bytes) {
                        replacement.push(b'\n');
                    }
                    splice_bytes(data, start..end, &replacement);
                    let delta_len = replacement.len() as isize - (end - start) as isize;
                    stream_offsets
                        .entry(info.stream_index)
                        .and_modify(|value| *value += delta_len)
                        .or_insert(delta_len);
                }
                _ => bail!("target {} not found", target.id),
            }
        }
    }

    if touched_streams.is_empty() {
        return Ok(original.to_vec());
    }

    let mut new_objects = Vec::new();
    let mut next_id = doc.max_id + 1;
    let mut content_refs = Vec::new();

    for (index, stream_entry) in page_cache.contents.streams.iter().enumerate() {
        if touched_streams.contains(&index) {
            let data = stream_data
                .remove(&index)
                .unwrap_or_else(|| stream_entry.data.clone());
            let original_stream = doc
                .get_object(stream_entry.id)
                .context("missing original stream")?
                .as_stream()
                .context("content object is not a stream")?;
            let mut dict = original_stream.dict.clone();
            dict.remove(b"Filter");
            dict.remove(b"DecodeParms");
            let stream = Stream::new(dict, data);
            let new_id: ObjectId = (next_id, 0);
            next_id += 1;
            new_objects.push(SerializedObject {
                id: new_id,
                object: Object::Stream(stream),
            });
            content_refs.push(Object::Reference(new_id));
        } else {
            content_refs.push(Object::Reference(stream_entry.id));
        }
    }

    let mut page_dict = doc
        .get_object(page_cache.page_id)
        .context("page object missing")?
        .as_dict()
        .context("page is not a dictionary")?
        .clone();
    let contents_object = if page_cache.contents.is_array {
        Object::Array(content_refs)
    } else {
        content_refs.into_iter().next().unwrap_or(Object::Null)
    };
    page_dict.set(b"Contents", contents_object);
    new_objects.push(SerializedObject {
        id: page_cache.page_id,
        object: Object::Dictionary(page_dict),
    });

    let final_max = new_objects
        .iter()
        .map(|obj| obj.id.0)
        .chain(std::iter::once(doc.max_id))
        .max()
        .unwrap_or(doc.max_id);
    trailer.set(b"Size", (final_max + 1) as i64);

    let updated = incremental_update(IncrementalWrite {
        original,
        trailer,
        prev_startxref,
        objects: new_objects,
    })?;

    Ok(updated)
}

fn matrix_multiply(a: [f64; 6], b: [f64; 6]) -> [f64; 6] {
    [
        a[0] * b[0] + a[2] * b[1],
        a[1] * b[0] + a[3] * b[1],
        a[0] * b[2] + a[2] * b[3],
        a[1] * b[2] + a[3] * b[3],
        a[0] * b[4] + a[2] * b[5] + a[4],
        a[1] * b[4] + a[3] * b[5] + a[5],
    ]
}

fn matrix_token(matrix: [f64; 6], operator: &str) -> String {
    let parts: Vec<String> = matrix.iter().map(|v| format_number(*v)).collect();
    format!(
        "{} {} {} {} {} {} {}",
        parts[0], parts[1], parts[2], parts[3], parts[4], parts[5], operator
    )
}

fn format_number(value: f64) -> String {
    let mut s = format!("{:.6}", value);
    while s.contains('.') && s.ends_with('0') {
        s.pop();
    }
    if s.ends_with('.') {
        s.pop();
    }
    if s.is_empty() || s == "-0" {
        "0".into()
    } else {
        s
    }
}

fn splice_bytes(data: &mut Vec<u8>, range: std::ops::Range<usize>, replacement: &[u8]) {
    data.splice(range, replacement.iter().copied());
}

fn find_startxref(bytes: &[u8]) -> Result<u64> {
    let needle = b"startxref";
    if let Some(pos) = bytes
        .windows(needle.len())
        .rposition(|window| window == needle)
    {
        let mut idx = pos + needle.len();
        while idx < bytes.len() && is_whitespace(bytes[idx]) {
            idx += 1;
        }
        let start = idx;
        while idx < bytes.len() && bytes[idx].is_ascii_digit() {
            idx += 1;
        }
        let value = std::str::from_utf8(&bytes[start..idx])?
            .trim()
            .parse::<u64>()?;
        Ok(value)
    } else {
        bail!("startxref not found");
    }
}

fn is_whitespace(byte: u8) -> bool {
    matches!(byte, b' ' | b'\t' | b'\r' | b'\n' | 0x0c | 0x00)
}

fn ends_with_whitespace(bytes: &[u8]) -> bool {
    bytes
        .last()
        .map(|b| matches!(b, b' ' | b'\t' | b'\r' | b'\n'))
        .unwrap_or(false)
}
