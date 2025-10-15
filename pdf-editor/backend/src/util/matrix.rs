/// Minimal affine matrix helpers shared across future patch logic.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Matrix2D(pub [f64; 6]);

impl Matrix2D {
    pub fn identity() -> Self {
        Self([1.0, 0.0, 0.0, 1.0, 0.0, 0.0])
    }
}
