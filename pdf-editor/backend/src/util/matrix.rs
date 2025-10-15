#[derive(Clone, Copy, Debug, Default)]
pub struct Matrix2D(pub [f32; 6]);

impl Matrix2D {
    pub fn identity() -> Self {
        Self([1.0, 0.0, 0.0, 1.0, 0.0, 0.0])
    }

    pub fn multiply(self, other: Self) -> Self {
        let [a1, b1, c1, d1, e1, f1] = self.0;
        let [a2, b2, c2, d2, e2, f2] = other.0;
        let a = a1 * a2 + b1 * c2;
        let b = a1 * b2 + b1 * d2;
        let c = c1 * a2 + d1 * c2;
        let d = c1 * b2 + d1 * d2;
        let e = e1 * a2 + f1 * c2 + e2;
        let f = e1 * b2 + f1 * d2 + f2;
        Matrix2D([a, b, c, d, e, f])
    }
}
