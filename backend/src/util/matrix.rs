#[derive(Debug, Clone, Copy, Default)]
pub struct Matrix2D {
    pub a: f32,
    pub b: f32,
    pub c: f32,
    pub d: f32,
    pub e: f32,
    pub f: f32,
}

impl Matrix2D {
    pub fn identity() -> Self {
        Self {
            a: 1.0,
            d: 1.0,
            ..Self::default()
        }
    }
}
