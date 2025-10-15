//! Incremental PDF writer that appends a new content stream revision.

use std::{collections::BTreeMap, io::Write};

use anyhow::{anyhow, Context, Result};
use lopdf::{Document, Object, ObjectId};

use crate::pdf::extract::PageCache;

pub fn incremental_update(
    original: &[u8],
    document: &Document,
    page: &PageCache,
) -> Result<Vec<u8>> {
    let mut output = original.to_vec();

    let combined_stream = combine_streams(page)?;

    let size_object = document
        .trailer
        .get(b"Size")
        .ok_or_else(|| anyhow!("trailer missing /Size"))?;
    let size_value = as_i64(size_object)?;
    let new_stream_id = size_value as u32;
    let new_stream_obj = ObjectId {
        obj: new_stream_id,
        gen: 0,
    };

    let mut page_dict = document
        .get_object(page.page_ref)
        .with_context(|| format!("page object {:?} missing", page.page_ref))?
        .as_dict()
        .context("page object is not a dictionary")?
        .clone();
    page_dict.set(b"Contents", Object::Reference(new_stream_obj));

    let page_offset = output.len();
    write_object(&mut output, page.page_ref, Object::Dictionary(page_dict))?;

    let stream_offset = output.len();
    write_stream(&mut output, new_stream_obj, &combined_stream)?;

    let xref_offset = output.len();
    let mut entries = BTreeMap::new();
    entries.insert(page.page_ref.obj, (page.page_ref.gen, page_offset));
    entries.insert(new_stream_id, (0u16, stream_offset));
    write_xref(&mut output, entries)?;

    let prev_offset = find_startxref(original)?;
    let mut trailer = document.trailer.clone();
    trailer.set(b"Size", Object::Integer((new_stream_id as i64) + 1));
    trailer.set(b"Prev", Object::Integer(prev_offset as i64));

    write!(output, "trailer\n")?;
    write!(output, "{}\n", Object::Dictionary(trailer).to_string())?;
    write!(output, "startxref\n{xref_offset}\n%%EOF\n")?;

    Ok(output)
}

fn combine_streams(page: &PageCache) -> Result<Vec<u8>> {
    let mut buffer = Vec::new();
    for stream_id in &page.order {
        let stream = page
            .streams
            .get(stream_id)
            .ok_or_else(|| anyhow!("missing stream {stream_id}"))?;
        buffer.extend_from_slice(&stream.bytes);
        if !stream.bytes.ends_with(b"\n") {
            buffer.push(b'\n');
        }
    }
    Ok(buffer)
}

fn write_object(buffer: &mut Vec<u8>, id: ObjectId, object: Object) -> Result<()> {
    write!(buffer, "{} {} obj\n", id.obj, id.gen)?;
    write!(buffer, "{}\n", object.to_string())?;
    write!(buffer, "endobj\n")?;
    Ok(())
}

fn write_stream(buffer: &mut Vec<u8>, id: ObjectId, content: &[u8]) -> Result<()> {
    write!(buffer, "{} {} obj\n", id.obj, id.gen)?;
    write!(buffer, "<< /Length {} >>\nstream\n", content.len())?;
    buffer.extend_from_slice(content);
    if !content.ends_with(b"\n") {
        buffer.push(b'\n');
    }
    buffer.extend_from_slice(b"endstream\nendobj\n");
    Ok(())
}

fn write_xref(buffer: &mut Vec<u8>, entries: BTreeMap<u32, (u16, usize)>) -> Result<()> {
    buffer.extend_from_slice(b"xref\n");
    let mut iter = entries.into_iter().peekable();
    while let Some((start_obj, (gen, offset))) = iter.next() {
        let mut count = 1usize;
        let mut offsets = vec![(gen, offset)];
        let mut next_expected = start_obj + 1;
        while let Some(&(next_obj, (next_gen, next_off))) = iter.peek() {
            if next_obj == next_expected {
                offsets.push((next_gen, next_off));
                count += 1;
                next_expected += 1;
                iter.next();
            } else {
                break;
            }
        }
        write!(buffer, "{} {}\n", start_obj, count)?;
        for (gen, off) in offsets {
            write!(buffer, "{:010} {:05} n \n", off, gen)?;
        }
    }
    Ok(())
}

fn find_startxref(bytes: &[u8]) -> Result<usize> {
    let needle = b"startxref";
    let position = bytes
        .windows(needle.len())
        .rposition(|window| window == needle)
        .ok_or_else(|| anyhow!("startxref not found"))?;
    let mut idx = position + needle.len();
    while idx < bytes.len() && bytes[idx].is_ascii_whitespace() {
        idx += 1;
    }
    let start = idx;
    while idx < bytes.len() && bytes[idx].is_ascii_digit() {
        idx += 1;
    }
    let number = std::str::from_utf8(&bytes[start..idx])?.parse::<usize>()?;
    Ok(number)
}

fn as_i64(object: &Object) -> Result<i64> {
    match object {
        Object::Integer(value) => Ok(*value),
        Object::Real(value) => Ok(*value as i64),
        _ => Err(anyhow!("expected integer")),
    }
}
