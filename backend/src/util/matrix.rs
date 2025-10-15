#[derive(Debug, Clone, Copy, Default)]
pub struct Matrix(pub [f32; 6]);

impl Matrix {
    pub fn identity() -> Self {
        Self([1.0, 0.0, 0.0, 1.0, 0.0, 0.0])
    }
}
