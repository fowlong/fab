//! 2D affine matrix helper utilities.

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Matrix(pub [f32; 6]);

impl Matrix {
    pub fn identity() -> Self {
        Matrix([1.0, 0.0, 0.0, 1.0, 0.0, 0.0])
    }

    pub fn multiply(&self, other: &Matrix) -> Matrix {
        let [a1, b1, c1, d1, e1, f1] = self.0;
        let [a2, b2, c2, d2, e2, f2] = other.0;
        Matrix([
            a1 * a2 + c1 * b2,
            b1 * a2 + d1 * b2,
            a1 * c2 + c1 * d2,
            b1 * c2 + d1 * d2,
            a1 * e2 + c1 * f2 + e1,
            b1 * e2 + d1 * f2 + f1,
        ])
    }

    pub fn invert(&self) -> Option<Matrix> {
        let [a, b, c, d, e, f] = self.0;
        let det = a * d - b * c;
        if det.abs() < f32::EPSILON {
            return None;
        }
        let inv_det = 1.0 / det;
        let na = d * inv_det;
        let nb = -b * inv_det;
        let nc = -c * inv_det;
        let nd = a * inv_det;
        let ne = -(na * e + nc * f);
        let nf = -(nb * e + nd * f);
        Some(Matrix([na, nb, nc, nd, ne, nf]))
    }
}
