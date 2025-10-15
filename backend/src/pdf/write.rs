use std::collections::BTreeMap;

use anyhow::Result;
use lopdf::{Dictionary, Object, ObjectId, StringFormat};

pub struct SerializedObject {
    pub id: ObjectId,
    pub object: Object,
}

pub struct IncrementalWrite<'a> {
    pub original: &'a [u8],
    pub trailer: Dictionary,
    pub prev_startxref: u64,
    pub objects: Vec<SerializedObject>,
}

pub fn incremental_update(mut write: IncrementalWrite<'_>) -> Result<Vec<u8>> {
    let mut output = Vec::with_capacity(write.original.len() + 1024);
    output.extend_from_slice(write.original);
    if !output.ends_with(b"\n") {
        output.push(b'\n');
    }

    let mut entries = BTreeMap::<u32, (u16, u64)>::new();

    for serialized in &write.objects {
        let offset = output.len() as u64;
        let bytes = serialize_object(serialized.id, &serialized.object)?;
        output.extend_from_slice(&bytes);
        if !output.ends_with(b"\n") {
            output.push(b'\n');
        }
        entries.insert(serialized.id.0, (serialized.id.1, offset));
    }

    let xref_start = output.len() as u64;
    output.extend_from_slice(b"xref\n");

    // Group entries by contiguous object ids.
    let mut iter = entries.into_iter().peekable();
    while let Some((start_obj, (gen, offset))) = iter.next() {
        let mut run = vec![(start_obj, gen, offset)];
        while let Some((next_obj, (next_gen, next_offset))) = iter.peek() {
            if *next_obj == run.last().unwrap().0 + 1 {
                let (obj, gen, off) = iter.next().unwrap();
                run.push((obj, gen, off));
            } else {
                break;
            }
        }
        output.extend_from_slice(format!("{} {}\n", run[0].0, run.len()).as_bytes());
        for (_, generation, off) in run {
            output.extend_from_slice(format!("{:010} {:05} n \n", off, generation).as_bytes());
        }
    }

    if write.prev_startxref > 0 {
        write.trailer.set(b"Prev", write.prev_startxref as i64);
    }

    output.extend_from_slice(b"trailer\n");
    let trailer_bytes = object_to_bytes(&Object::Dictionary(write.trailer))?;
    output.extend_from_slice(&trailer_bytes);
    output.push(b'\n');
    output.extend_from_slice(b"startxref\n");
    output.extend_from_slice(format!("{}\n", xref_start).as_bytes());
    output.extend_from_slice(b"%%EOF\n");

    Ok(output)
}

fn serialize_object(id: ObjectId, object: &Object) -> Result<Vec<u8>> {
    let mut out = Vec::new();
    out.extend_from_slice(format!("{} {} obj\n", id.0, id.1).as_bytes());
    match object {
        Object::Stream(stream) => {
            let mut stream = stream.clone();
            stream.dict.set(b"Length", stream.content.len() as i64);
            let dict_bytes = object_to_bytes(&Object::Dictionary(stream.dict.clone()))?;
            out.extend_from_slice(&dict_bytes);
            out.extend_from_slice(b"\nstream\n");
            out.extend_from_slice(&stream.content);
            if !stream.content.ends_with(b"\n") {
                out.push(b'\n');
            }
            out.extend_from_slice(b"endstream");
        }
        other => {
            let bytes = object_to_bytes(other)?;
            out.extend_from_slice(&bytes);
        }
    }
    out.extend_from_slice(b"\nendobj\n");
    Ok(out)
}

fn object_to_bytes(object: &Object) -> Result<Vec<u8>> {
    let mut out = Vec::new();
    write_object(object, &mut out)?;
    Ok(out)
}

fn write_object(object: &Object, out: &mut Vec<u8>) -> Result<()> {
    match object {
        Object::Null => out.extend_from_slice(b"null"),
        Object::Boolean(value) => {
            if *value {
                out.extend_from_slice(b"true")
            } else {
                out.extend_from_slice(b"false")
            }
        }
        Object::Integer(value) => out.extend_from_slice(format!("{}", value).as_bytes()),
        Object::Real(value) => out.extend_from_slice(format_float(*value).as_bytes()),
        Object::Name(name) => {
            out.push(b'/');
            out.extend_from_slice(name);
        }
        Object::String(text, format) => match format {
            StringFormat::Literal => {
                out.push(b'(');
                out.extend(text);
                out.push(b')');
            }
            StringFormat::Hexadecimal => {
                out.push(b'<');
                for byte in text {
                    out.extend_from_slice(format!("{:02x}", byte).as_bytes());
                }
                out.push(b'>');
            }
        },
        Object::Array(items) => {
            out.push(b'[');
            let mut first = true;
            for item in items {
                if !first {
                    out.push(b' ');
                }
                first = false;
                write_object(item, out)?;
            }
            out.push(b']');
        }
        Object::Dictionary(dict) => {
            out.extend_from_slice(b"<<");
            for (key, value) in dict.iter() {
                out.push(b'/');
                out.extend_from_slice(key);
                out.push(b' ');
                write_object(value, out)?;
                out.push(b' ');
            }
            out.extend_from_slice(b">>");
        }
        Object::Reference(id) => {
            out.extend_from_slice(format!("{} {} R", id.0, id.1).as_bytes());
        }
        Object::Stream(stream) => {
            let mut dict_bytes = object_to_bytes(&Object::Dictionary(stream.dict.clone()))?;
            out.append(&mut dict_bytes);
            out.extend_from_slice(b"\nstream\n");
            out.extend_from_slice(&stream.content);
            if !stream.content.ends_with(b"\n") {
                out.push(b'\n');
            }
            out.extend_from_slice(b"endstream");
        }
    }
    Ok(())
}

fn format_float(value: f64) -> String {
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
