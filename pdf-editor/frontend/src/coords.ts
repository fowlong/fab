export const POINTS_PER_PIXEL = 72 / 96;

export type Matrix = [number, number, number, number, number, number];

export function multiply(m1: Matrix, m2: Matrix): Matrix {
  const [a1, b1, c1, d1, e1, f1] = m1;
  const [a2, b2, c2, d2, e2, f2] = m2;
  return [
    a1 * a2 + c1 * b2,
    b1 * a2 + d1 * b2,
    a1 * c2 + c1 * d2,
    b1 * c2 + d1 * d2,
    a1 * e2 + c1 * f2 + e1,
    b1 * e2 + d1 * f2 + f1,
  ];
}

export function inverse(m: Matrix): Matrix {
  const [a, b, c, d, e, f] = m;
  const det = a * d - b * c;
  if (Math.abs(det) < 1e-12) {
    throw new Error("Matrix not invertible");
  }
  const invDet = 1 / det;
  const na = d * invDet;
  const nb = -b * invDet;
  const nc = -c * invDet;
  const nd = a * invDet;
  const ne = -(na * e + nc * f);
  const nf = -(nb * e + nd * f);
  return [na, nb, nc, nd, ne, nf];
}

export function pxToPtMatrix(pageHeightPt: number): Matrix {
  return [POINTS_PER_PIXEL, 0, 0, -POINTS_PER_PIXEL, 0, pageHeightPt];
}

export function ptToPxMatrix(pageHeightPt: number): Matrix {
  return inverse(pxToPtMatrix(pageHeightPt));
}

export function fabricDeltaToPdfDelta(
  fold: Matrix,
  fnew: Matrix,
  pageHeightPt: number,
): Matrix {
  const deltaFabric = multiply(fnew, inverse(fold));
  const pxToPt = pxToPtMatrix(pageHeightPt);
  const ptToPx = inverse(pxToPt);
  return multiply(multiply(pxToPt, deltaFabric), ptToPx);
}

export function applyToPoint(m: Matrix, x: number, y: number): [number, number] {
  const [a, b, c, d, e, f] = m;
  return [a * x + c * y + e, b * x + d * y + f];
}
