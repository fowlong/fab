#[derive(Debug, Clone, Copy)]
pub struct Matrix(pub [f64; 6]);

impl Matrix {
    pub fn identity() -> Self {
        Self([1.0, 0.0, 0.0, 1.0, 0.0, 0.0])
    }
}
