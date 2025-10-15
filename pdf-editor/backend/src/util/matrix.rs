#[derive(Debug, Clone, Copy)]
pub struct Matrix([f32; 6]);

impl Matrix {
    pub fn identity() -> Self {
        Matrix([1.0, 0.0, 0.0, 1.0, 0.0, 0.0])
    }
}
