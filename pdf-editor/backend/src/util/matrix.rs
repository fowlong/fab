#![allow(dead_code)]

use crate::types::PdfMatrix;

pub fn multiply(a: PdfMatrix, b: PdfMatrix) -> PdfMatrix {
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

pub fn identity() -> PdfMatrix {
    [1.0, 0.0, 0.0, 1.0, 0.0, 0.0]
}
