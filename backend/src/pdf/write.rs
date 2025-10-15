//! Incremental PDF writer for transform updates.

use std::io::Write;

use anyhow::{anyhow, Context, Result};
use lopdf::{Dictionary, Document, Object, Stream};

use super::patch::PagePatch;

pub fn incremental_update(
    doc: &mut Document,
    base_bytes: &[u8],
    patch: PagePatch,
) -> Result<Vec<u8>> {
    let prev_offset = find_startxref(base_bytes)?;

    let new_stream_id = doc.max_id + 1;
    doc.max_id = new_stream_id;

    let mut stream_dict = Dictionary::new();
    stream_dict.set(
        b"Length",
        Object::Integer(patch.new_stream_bytes.len() as i64),
    );
    let mut stream = Stream::new(stream_dict, patch.new_stream_bytes.clone());
    stream.set_plain_content(patch.new_stream_bytes.clone());

    let new_stream_obj = Object::Stream(stream.clone());
    doc.objects
        .insert((new_stream_id, 0), new_stream_obj.clone());

    let mut page_dict = doc
        .get_object(patch.page_id)
        .and_then(|obj| obj.as_dict().cloned())
        .ok()
        .context("page dictionary missing during incremental update")?;
    page_dict.set(b"Contents", Object::Reference((new_stream_id, 0)));
    let page_object = Object::Dictionary(page_dict.clone());
    doc.objects.insert(patch.page_id, page_object.clone());

    let mut appended = Vec::new();
    let base_offset = base_bytes.len();
    let mut offsets: Vec<((u32, u16), usize)> = Vec::new();

    let stream_offset = write_object(
        &mut appended,
        base_offset,
        (new_stream_id, 0),
        &new_stream_obj,
    )?;
    offsets.push(((new_stream_id, 0), stream_offset));

    let page_offset = write_object(&mut appended, base_offset, patch.page_id, &page_object)?;
    offsets.push((patch.page_id, page_offset));

    let xref_offset = base_offset + appended.len();
    write_xref(&mut appended, &offsets)?;

    let mut trailer = doc.trailer.clone();
    let size = doc.max_id as i64 + 1;
    trailer.set(b"Size", Object::Integer(size));
    trailer.set(b"Prev", Object::Integer(prev_offset as i64));

    appended.extend_from_slice(b"trailer\n");
    Object::Dictionary(trailer.clone()).to_pdf(&mut appended)?;
    appended.extend_from_slice(b"\nstartxref\n");
    writeln!(appended, "{xref_offset}")?;
    appended.extend_from_slice(b"%%EOF\n");

    doc.trailer = trailer;

    let mut output = Vec::with_capacity(base_bytes.len() + appended.len());
    output.extend_from_slice(base_bytes);
    output.extend_from_slice(&appended);
    Ok(output)
}

fn write_object(
    buffer: &mut Vec<u8>,
    base_offset: usize,
    id: (u32, u16),
    object: &Object,
) -> Result<usize> {
    let offset = base_offset + buffer.len();
    writeln!(buffer, "{} {} obj", id.0, id.1)?;
    object.to_pdf(buffer)?;
    buffer.extend_from_slice(b"\nendobj\n");
    Ok(offset)
}

fn write_xref(buffer: &mut Vec<u8>, entries: &[((u32, u16), usize)]) -> Result<()> {
    buffer.extend_from_slice(b"xref\n");
    let mut sorted = entries.to_vec();
    sorted.sort_by_key(|(id, _)| (id.0, id.1));
    for ((obj, gen), offset) in sorted {
        writeln!(buffer, "{} 1", obj)?;
        writeln!(buffer, "{:010} {:05} n ", offset, gen)?;
    }
    Ok(())
}

fn find_startxref(bytes: &[u8]) -> Result<usize> {
    let marker = b"startxref";
    let position = bytes
        .windows(marker.len())
        .rposition(|window| window == marker)
        .ok_or_else(|| anyhow!("PDF missing startxref marker"))?;
    let mut index = position + marker.len();
    while index < bytes.len() && bytes[index].is_ascii_whitespace() {
        index += 1;
    }
    let mut end = index;
    while end < bytes.len() && bytes[end].is_ascii_digit() {
        end += 1;
    }
    let value = std::str::from_utf8(&bytes[index..end])?
        .trim()
        .parse::<usize>()?;
    Ok(value)
}
