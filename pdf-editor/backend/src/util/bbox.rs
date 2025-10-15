#[derive(Debug, Clone, Copy)]
pub struct BoundingBox {
    pub x0: f32,
    pub y0: f32,
    pub x1: f32,
    pub y1: f32,
}

impl BoundingBox {
    pub fn empty() -> Self {
        Self {
            x0: 0.0,
            y0: 0.0,
            x1: 0.0,
            y1: 0.0,
        }
    }
}
