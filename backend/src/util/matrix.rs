use crate::types::Matrix;

/// Represents a 2D affine matrix in PDF order [a, b, c, d, e, f].
#[allow(dead_code)]
pub fn identity() -> Matrix {
    Matrix([1.0, 0.0, 0.0, 1.0, 0.0, 0.0])
}

#[allow(dead_code)]
pub fn multiply(lhs: &Matrix, rhs: &Matrix) -> Matrix {
    let [a1, b1, c1, d1, e1, f1] = lhs.0;
    let [a2, b2, c2, d2, e2, f2] = rhs.0;
    Matrix([
        a1 * a2 + b1 * c2,
        a1 * b2 + b1 * d2,
        c1 * a2 + d1 * c2,
        c1 * b2 + d1 * d2,
        e1 * a2 + f1 * c2 + e2,
        e1 * b2 + f1 * d2 + f2,
    ])
}

#[allow(dead_code)]
pub fn invert(m: &Matrix) -> Option<Matrix> {
    let [a, b, c, d, e, f] = m.0;
    let det = a * d - b * c;
    if det.abs() < f32::EPSILON {
        return None;
    }
    let inv_det = 1.0 / det;
    let a_inv = d * inv_det;
    let b_inv = -b * inv_det;
    let c_inv = -c * inv_det;
    let d_inv = a * inv_det;
    let e_inv = -(a_inv * e + c_inv * f);
    let f_inv = -(b_inv * e + d_inv * f);
    Some(Matrix([a_inv, b_inv, c_inv, d_inv, e_inv, f_inv]))
}
