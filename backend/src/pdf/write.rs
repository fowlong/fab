//! Incremental PDF writing utilities.

use std::collections::HashMap;
use std::io::Write;

use anyhow::{anyhow, Result};
use lopdf::{Object, ObjectId};

use super::loader::LoadedDoc;

pub fn incremental_update(
    doc: &LoadedDoc,
    updates: &HashMap<ObjectId, Vec<u8>>,
) -> Result<Vec<u8>> {
    if updates.is_empty() {
        return Ok(doc.bytes.clone());
    }

    let mut output = doc.bytes.clone();
    let mut object_chunks: Vec<(ObjectId, Vec<u8>)> = Vec::new();
    let mut replacements: HashMap<ObjectId, ObjectId> = HashMap::new();
    let mut next_id = doc.doc.max_id + 1;
    let mut max_obj_id = doc.doc.max_id;

    for stream in &doc.cache.streams {
        if let Some(new_bytes) = updates.get(&stream.object_id) {
            let mut dict = stream.dict.clone();
            dict.set(b"Length", Object::Integer(new_bytes.len() as i64));
            dict.remove(b"Filter");
            dict.remove(b"DecodeParms");

            let object_id = (next_id, 0);
            max_obj_id = max_obj_id.max(object_id.0);
            next_id += 1;
            let mut chunk = Vec::new();
            write!(&mut chunk, "{} {} obj\n", object_id.0, object_id.1)?;
            Object::Dictionary(dict).to_pdf(&mut chunk)?;
            chunk.extend_from_slice(b"\nstream\n");
            chunk.extend_from_slice(new_bytes);
            if !new_bytes.ends_with(b"\n") {
                chunk.push(b'\n');
            }
            chunk.extend_from_slice(b"endstream\nendobj\n");
            object_chunks.push((object_id, chunk));
            replacements.insert(stream.object_id, object_id);
        }
    }

    if object_chunks.is_empty() {
        return Ok(doc.bytes.clone());
    }

    let mut page_dict = doc
        .doc
        .get_object(doc.cache.page_id)
        .ok()
        .and_then(|obj| obj.as_dict().ok())
        .cloned()
        .ok_or_else(|| anyhow!("page dictionary missing"))?;

    let mut contents_objects = Vec::new();
    for stream in &doc.cache.streams {
        if let Some(new_id) = replacements.get(&stream.object_id) {
            contents_objects.push(Object::Reference(*new_id));
        } else {
            contents_objects.push(Object::Reference(stream.object_id));
        }
    }

    let contents_value = if contents_objects.len() == 1 {
        contents_objects.into_iter().next().unwrap()
    } else {
        Object::Array(contents_objects)
    };
    page_dict.set(b"Contents", contents_value);

    let page_chunk = {
        let mut chunk = Vec::new();
        write!(
            &mut chunk,
            "{} {} obj\n",
            doc.cache.page_id.0, doc.cache.page_id.1
        )?;
        Object::Dictionary(page_dict).to_pdf(&mut chunk)?;
        chunk.extend_from_slice(b"\nendobj\n");
        (doc.cache.page_id, chunk)
    };

    max_obj_id = max_obj_id.max(page_chunk.0 .0);
    object_chunks.push(page_chunk);
    object_chunks.sort_by_key(|(id, _)| id.0);

    let base_offset = output.len() as u64;
    let mut offsets = Vec::new();
    let mut body = Vec::new();
    for (id, chunk) in &object_chunks {
        offsets.push((*id, base_offset + body.len() as u64));
        body.extend_from_slice(chunk);
    }

    output.extend_from_slice(&body);

    let xref_start = output.len() as u64;
    let mut xref = Vec::new();
    xref.extend_from_slice(b"xref\n");
    let mut i = 0;
    while i < offsets.len() {
        let start_obj = offsets[i].0 .0;
        let mut j = i + 1;
        while j < offsets.len() && offsets[j].0 .0 == offsets[j - 1].0 .0 + 1 {
            j += 1;
        }
        let count = j - i;
        write!(&mut xref, "{} {}\n", start_obj, count)?;
        for k in i..j {
            let (obj_id, offset) = offsets[k];
            write!(&mut xref, "{:010} {:05} n \n", offset, obj_id.1)?;
        }
        i = j;
    }

    output.extend_from_slice(&xref);

    let mut trailer = doc.doc.trailer.clone();
    trailer.set(b"Size", Object::Integer((max_obj_id as i64) + 1));
    trailer.set(b"Prev", Object::Integer(doc.startxref as i64));

    output.extend_from_slice(b"trailer\n");
    Object::Dictionary(trailer).to_pdf(&mut output)?;
    output.extend_from_slice(b"\nstartxref\n");
    write!(&mut output, "{}\n%%EOF\n", xref_start)?;

    Ok(output)
}
