use crate::types::BoundingBox;

#[allow(dead_code)]
pub fn empty() -> BoundingBox {
    BoundingBox([0.0, 0.0, 0.0, 0.0])
}

#[allow(dead_code)]
pub fn expand(bbox: &mut BoundingBox, x: f32, y: f32) {
    let [x0, y0, x1, y1] = bbox.0;
    let new_x0 = x0.min(x);
    let new_y0 = y0.min(y);
    let new_x1 = x1.max(x);
    let new_y1 = y1.max(y);
    *bbox = BoundingBox([new_x0, new_y0, new_x1, new_y1]);
}
