use crate::types::Matrix;

pub fn multiply(a: Matrix, b: Matrix) -> Matrix {
    let [a1, b1, c1, d1, e1, f1] = a;
    let [a2, b2, c2, d2, e2, f2] = b;
    [
        a1 * a2 + c1 * b2,
        b1 * a2 + d1 * b2,
        a1 * c2 + c1 * d2,
        b1 * c2 + d1 * d2,
        a1 * e2 + c1 * f2 + e1,
        b1 * e2 + d1 * f2 + f1,
    ]
}

pub fn invert(m: Matrix) -> Option<Matrix> {
    let [a, b, c, d, e, f] = m;
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
