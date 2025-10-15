//! Incremental PDF writer for appending updated objects.

use std::collections::BTreeMap;
use std::io::Write;

use anyhow::{anyhow, Result};
use lopdf::{Dictionary, Object, ObjectId};

pub struct IncrementalWrite {
    pub bytes: Vec<u8>,
    pub trailer: Dictionary,
}

pub fn incremental_update(
    original: &[u8],
    trailer: &Dictionary,
    updates: &[(ObjectId, Object)],
) -> Result<IncrementalWrite> {
    if updates.is_empty() {
        return Ok(IncrementalWrite {
            bytes: original.to_vec(),
            trailer: trailer.clone(),
        });
    }

    let prev_startxref = find_startxref(original)?;
    let mut trailer_dict = trailer.clone();
    let mut size = trailer_dict
        .get(b"Size")
        .and_then(|obj| obj.as_i64().ok())
        .unwrap_or(0) as u32;

    let mut ordered: BTreeMap<ObjectId, Object> = BTreeMap::new();
    for (id, object) in updates {
        ordered.insert(*id, object.clone());
        if id.0 + 1 > size {
            size = id.0 + 1;
        }
    }

    trailer_dict.set(b"Size", Object::Integer(size as i64));
    trailer_dict.set(b"Prev", Object::Integer(prev_startxref as i64));

    let mut body = Vec::new();
    let mut offsets: Vec<(ObjectId, usize)> = Vec::new();
    for (id, object) in ordered.iter() {
        let offset = original.len() + body.len();
        offsets.push((*id, offset));
        write_object(&mut body, *id, object)?;
    }

    let xref_offset = original.len() + body.len();
    let mut xref = Vec::new();
    xref.extend_from_slice(b"xref\n");

    if !offsets.is_empty() {
        let mut current_start = offsets[0].0;
        let mut current_entries: Vec<(ObjectId, usize)> = Vec::new();
        let mut last_num = current_start.0;
        for (id, offset) in offsets {
            if current_entries.is_empty() {
                current_start = id;
                last_num = id.0;
            }
            if !current_entries.is_empty() && (id.0 != last_num + 1 || id.1 != current_start.1) {
                write_subsection(&mut xref, current_start, &current_entries)?;
                current_entries.clear();
                current_start = id;
                last_num = id.0;
            }
            current_entries.push((id, offset));
            last_num = id.0;
        }
        if !current_entries.is_empty() {
            write_subsection(&mut xref, current_start, &current_entries)?;
        }
    }

    xref.extend_from_slice(b"trailer\n");
    let mut trailer_bytes = Vec::new();
    Object::Dictionary(trailer_dict).to_pdf(&mut trailer_bytes)?;
    xref.extend_from_slice(&trailer_bytes);
    if !trailer_bytes.ends_with(b"\n") {
        xref.push(b'\n');
    }
    xref.extend_from_slice(b"startxref\n");
    writeln!(&mut xref, "{}", xref_offset)?;
    xref.extend_from_slice(b"%%EOF\n");

    let mut output = Vec::with_capacity(original.len() + body.len() + xref.len());
    output.extend_from_slice(original);
    output.extend_from_slice(&body);
    output.extend_from_slice(&xref);
    Ok(IncrementalWrite {
        bytes: output,
        trailer: trailer_dict,
    })
}

fn write_object(buffer: &mut Vec<u8>, id: ObjectId, object: &Object) -> Result<()> {
    write!(buffer, "{} {} obj\n", id.0, id.1)?;
    let mut object_bytes = Vec::new();
    object.to_pdf(&mut object_bytes)?;
    buffer.extend_from_slice(&object_bytes);
    if !object_bytes.ends_with(b"\n") {
        buffer.push(b'\n');
    }
    buffer.extend_from_slice(b"endobj\n");
    Ok(())
}

fn write_subsection(
    buffer: &mut Vec<u8>,
    start: ObjectId,
    entries: &[(ObjectId, usize)],
) -> Result<()> {
    writeln!(buffer, "{} {}", start.0, entries.len())?;
    for (id, offset) in entries {
        writeln!(buffer, "{:010} {:05} n ", offset, id.1)?;
    }
    Ok(())
}

pub fn find_startxref(bytes: &[u8]) -> Result<usize> {
    let marker = b"startxref";
    let position = bytes
        .windows(marker.len())
        .rposition(|window| window == marker)
        .ok_or_else(|| anyhow!("startxref marker not found"))?;
    let mut index = position + marker.len();
    while index < bytes.len() && bytes[index].is_ascii_whitespace() {
        index += 1;
    }
    let start = index;
    while index < bytes.len() && bytes[index].is_ascii_digit() {
        index += 1;
    }
    let value = std::str::from_utf8(&bytes[start..index])?.parse::<usize>()?;
    Ok(value)
}
*** End Patch
