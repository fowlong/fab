//! Minimal PDF content utilities used by the IR extraction stubs.

use lopdf::content::Content;

use super::Result;

#[derive(Debug, Clone)]
pub struct ContentSpan {
    pub stream_object: u32,
    pub start: usize,
    pub end: usize,
}

pub fn load_content(document: &lopdf::Document, stream_ref: lopdf::ObjectId) -> Result<Content> {
    let stream = document
        .get_object(stream_ref)?
        .as_stream()?
        .decompressed()?;
    Ok(Content::decode(&stream)?)
}

pub fn bbox_from_cm(cm: [f64; 6], width: f64, height: f64) -> ([f64; 4], [f64; 4]) {
    let matrix = crate::util::matrix::Matrix2D::from_array(cm);
    let points = [
        matrix.transform_point(0.0, 0.0),
        matrix.transform_point(width, 0.0),
        matrix.transform_point(0.0, height),
        matrix.transform_point(width, height),
    ];
    let min_x = points.iter().map(|p| p.0).fold(f64::INFINITY, f64::min);
    let max_x = points.iter().map(|p| p.0).fold(f64::NEG_INFINITY, f64::max);
    let min_y = points.iter().map(|p| p.1).fold(f64::INFINITY, f64::min);
    let max_y = points.iter().map(|p| p.1).fold(f64::NEG_INFINITY, f64::max);
    (
        [min_x, min_y, max_x, max_y],
        [cm[4], cm[5], cm[4] + width, cm[5] + height],
    )
}

pub fn replace_stream(
    document: &mut lopdf::Document,
    stream_ref: lopdf::ObjectId,
    data: Vec<u8>,
) -> Result<()> {
    let stream = document.get_object_mut(stream_ref)?.as_stream_mut()?;
    stream.set_plain_content(data);
    Ok(())
}

pub fn find_operator_span(content: &Content, operator: &str) -> Option<ContentSpan> {
    for operation in &content.operations {
        if operation.operator == operator {
            return Some(ContentSpan {
                stream_object: 0,
                start: 0,
                end: 0,
            });
        }
    }
    None
}
