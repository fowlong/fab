export const S = 72 / 96;

type Matrix = [number, number, number, number, number, number];

export function concat(m1: Matrix, m2: Matrix): Matrix {
  const [a1, b1, c1, d1, e1, f1] = m1;
  const [a2, b2, c2, d2, e2, f2] = m2;
  return [
    a1 * a2 + c1 * b2,
    b1 * a2 + d1 * b2,
    a1 * c2 + c1 * d2,
    b1 * c2 + d1 * d2,
    a1 * e2 + c1 * f2 + e1,
    b1 * e2 + d1 * f2 + f1
  ];
}

export function invert([a, b, c, d, e, f]: Matrix): Matrix {
  const det = a * d - b * c;
  if (Math.abs(det) < 1e-8) {
    throw new Error("Matrix not invertible");
  }
  const invDet = 1 / det;
  return [
    d * invDet,
    -b * invDet,
    -c * invDet,
    a * invDet,
    (c * f - d * e) * invDet,
    (b * e - a * f) * invDet
  ];
}

export function pxToPtMatrix(pageHeightPt: number): Matrix {
  return [S, 0, 0, -S, 0, pageHeightPt];
}

export function ptToPxMatrix(pageHeightPt: number): Matrix {
  return invert(pxToPtMatrix(pageHeightPt));
}

export function fabricDeltaToPdfDelta(fOld: Matrix, fNew: Matrix, pageHeightPt: number): Matrix {
  const deltaFabric = concat(fNew, invert(fOld));
  const pxToPt = pxToPtMatrix(pageHeightPt);
  const ptToPx = invert(pxToPt);
  return concat(concat(pxToPt, deltaFabric), ptToPx);
}
