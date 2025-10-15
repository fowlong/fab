//! 2D affine matrix helpers shared between the patch logic and IR extraction.

use crate::types::Matrix;

/// Multiply two 2D affine matrices (m1 * m2).
pub fn multiply(m1: &Matrix, m2: &Matrix) -> Matrix {
    let [a1, b1, c1, d1, e1, f1] = *m1;
    let [a2, b2, c2, d2, e2, f2] = *m2;
    [
        a1 * a2 + b1 * c2,
        a1 * b2 + b1 * d2,
        c1 * a2 + d1 * c2,
        c1 * b2 + d1 * d2,
        e1 * a2 + f1 * c2 + e2,
        e1 * b2 + f1 * d2 + f2,
    ]
}

/// Compute the inverse of a 2D affine matrix.
pub fn invert(m: &Matrix) -> Option<Matrix> {
    let [a, b, c, d, e, f] = *m;
    let det = a * d - b * c;
    if det.abs() < f64::EPSILON {
        return None;
    }
    let inv_det = 1.0 / det;
    let na = d * inv_det;
    let nb = -b * inv_det;
    let nc = -c * inv_det;
    let nd = a * inv_det;
    let ne = -(na * e + nc * f);
    let nf = -(nb * e + nd * f);
    Some([na, nb, nc, nd, ne, nf])
}

/// Apply an affine transform to a point (x, y).
pub fn apply_to_point(m: &Matrix, x: f64, y: f64) -> (f64, f64) {
    let [a, b, c, d, e, f] = *m;
    let nx = a * x + c * y + e;
    let ny = b * x + d * y + f;
    (nx, ny)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn multiply_identity() {
        let identity: Matrix = [1.0, 0.0, 0.0, 1.0, 0.0, 0.0];
        let m: Matrix = [0.5, 0.1, -0.1, 0.9, 10.0, -20.0];
        assert_eq!(multiply(&identity, &m), m);
        assert_eq!(multiply(&m, &identity), m);
    }

    #[test]
    fn invert_roundtrip() {
        let m: Matrix = [2.0, 0.0, 0.0, 2.0, 5.0, -3.0];
        let inv = invert(&m).unwrap();
        let id = multiply(&m, &inv);
        let expected: Matrix = [1.0, 0.0, 0.0, 1.0, 0.0, 0.0];
        for (lhs, rhs) in id.into_iter().zip(expected.into_iter()) {
            assert!((lhs - rhs).abs() < 1e-9);
        }
    }

    #[test]
    fn apply_point() {
        let m: Matrix = [1.0, 0.0, 0.0, 1.0, 5.0, -2.0];
        let (x, y) = apply_to_point(&m, 10.0, 5.0);
        assert_eq!((x, y), (15.0, 3.0));
    }
}
