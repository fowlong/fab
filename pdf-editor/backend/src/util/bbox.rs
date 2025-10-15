#[derive(Debug, Clone, Copy, Default)]
pub struct BBox {
    pub left: f32,
    pub bottom: f32,
    pub right: f32,
    pub top: f32,
}

impl BBox {
    pub fn from_points(points: &[(f32, f32)]) -> Option<Self> {
        if points.is_empty() {
            return None;
        }
        let (mut left, mut bottom) = points[0];
        let (mut right, mut top) = points[0];
        for &(x, y) in &points[1..] {
            left = left.min(x);
            right = right.max(x);
            bottom = bottom.min(y);
            top = top.max(y);
        }
        Some(Self { left, bottom, right, top })
    }
}
