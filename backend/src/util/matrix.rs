pub type Matrix = [f32; 6];

pub fn identity() -> Matrix {
    [1.0, 0.0, 0.0, 1.0, 0.0, 0.0]
}

pub fn multiply(a: Matrix, b: Matrix) -> Matrix {
    let [a0, a1, a2, a3, a4, a5] = a;
    let [b0, b1, b2, b3, b4, b5] = b;
    [
        a0 * b0 + a2 * b1,
        a1 * b0 + a3 * b1,
        a0 * b2 + a2 * b3,
        a1 * b2 + a3 * b3,
        a0 * b4 + a2 * b5 + a4,
        a1 * b4 + a3 * b5 + a5,
    ]
}

pub fn invert(m: Matrix) -> Option<Matrix> {
    let [a, b, c, d, e, f] = m;
    let det = a * d - b * c;
    if det.abs() <= f32::EPSILON {
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
