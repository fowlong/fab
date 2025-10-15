//! Simple 2D affine helpers used across the code base.

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Matrix(pub [f32; 6]);

impl Matrix {
    #[must_use]
    pub const fn identity() -> Self {
        Self([1.0, 0.0, 0.0, 1.0, 0.0, 0.0])
    }

    #[must_use]
    pub fn multiply(self, other: Self) -> Self {
        let [a1, b1, c1, d1, e1, f1] = self.0;
        let [a2, b2, c2, d2, e2, f2] = other.0;
        Self([
            a1 * a2 + b1 * c2,
            a1 * b2 + b1 * d2,
            c1 * a2 + d1 * c2,
            c1 * b2 + d1 * d2,
            e1 * a2 + f1 * c2 + e2,
            e1 * b2 + f1 * d2 + f2,
        ])
    }

    #[must_use]
    pub fn invert(self) -> Option<Self> {
        let [a, b, c, d, e, f] = self.0;
        let det = a * d - b * c;
        if det.abs() < f32::EPSILON {
            return None;
        }
        let inv_det = 1.0 / det;
        Some(Self([
            d * inv_det,
            -b * inv_det,
            -c * inv_det,
            a * inv_det,
            (c * f - d * e) * inv_det,
            (b * e - a * f) * inv_det,
        ]))
    }
}
