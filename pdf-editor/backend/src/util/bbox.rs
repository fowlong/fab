#[derive(Debug, Clone, Copy, Default)]
pub struct BBox {
    pub min_x: f32,
    pub min_y: f32,
    pub max_x: f32,
    pub max_y: f32,
}

impl BBox {
    pub fn new() -> Self {
        Self::default()
    }
}
