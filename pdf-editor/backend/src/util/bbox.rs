//! Axis-aligned bounding box helpers.

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BBox(pub [f32; 4]);

impl BBox {
    pub fn empty() -> Self {
        Self([0.0, 0.0, 0.0, 0.0])
    }
}
