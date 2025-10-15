use lopdf::content::Content;
use lopdf::Object;

pub fn merge_streams(streams: &[Object]) -> lopdf::Result<Content> {
    let mut combined = Content::new();
    for obj in streams {
        match obj {
            Object::Stream(stream) => {
                combined.operations.extend(Content::decode(&stream.content)?.operations);
            }
            _ => continue,
        }
    }
    Ok(combined)
}
