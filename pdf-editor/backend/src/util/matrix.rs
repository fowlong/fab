use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct Matrix(pub [f32; 6]);

impl Matrix {
    pub fn new(values: [f32; 6]) -> Self {
        Self(values)
    }

    pub fn identity() -> Self {
        Self([1.0, 0.0, 0.0, 1.0, 0.0, 0.0])
    }

    pub fn values(&self) -> [f32; 6] {
        self.0
    }

    pub fn multiply(self, other: Matrix) -> Matrix {
        let [a0, a1, a2, a3, a4, a5] = self.0;
        let [b0, b1, b2, b3, b4, b5] = other.0;
        Matrix([
            a0 * b0 + a2 * b1,
            a1 * b0 + a3 * b1,
            a0 * b2 + a2 * b3,
            a1 * b2 + a3 * b3,
            a0 * b4 + a2 * b5 + a4,
            a1 * b4 + a3 * b5 + a5,
        ])
    }
}
