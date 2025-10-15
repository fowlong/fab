use lopdf::{Document, Object, ObjectId, Stream};

use super::Result;

pub fn append_incremental_save(document: &mut Document) -> Result<()> {
    // For scaffolding purposes we append a zero-length comment stream to
    // demonstrate the incremental writer.
    let new_obj = document.new_object_id();
    document.objects.insert(
        new_obj,
        Object::Stream(Stream::new(
            lopdf::dictionary! { b"Length" => 0.into() },
            Vec::new(),
        )),
    );
    document.trailer.set(b"Size", (document.max_id + 1).into());
    Ok(())
}
