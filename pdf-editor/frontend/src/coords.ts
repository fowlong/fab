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
  if (Math.abs(det) < 1e-9) {
    throw new Error("Matrix not invertible");
  }
  const invDet = 1 / det;
  return [
    d * invDet,
    -b * invDet,
    -c * invDet,
    a * invDet,
    (c * f - d * e) * invDet,
    (b * e - a * f) * invDet,
  ];
}

export function pxToPtMatrix(pageHeightPt: number): Matrix {
  return [POINTS_PER_PIXEL, 0, 0, -POINTS_PER_PIXEL, 0, pageHeightPt];
}

export function fabricDeltaToPdfDelta(
  oldMatrix: Matrix,
  newMatrix: Matrix,
  pageHeightPt: number
): Matrix {
  const deltaFabric = multiply(newMatrix, inverse(oldMatrix));
  const pxToPt = pxToPtMatrix(pageHeightPt);
  const ptToPx = inverse(pxToPt);
  return multiply(multiply(pxToPt, deltaFabric), ptToPx);
}

export function applyMatrixToPoint(
  matrix: Matrix,
  point: { x: number; y: number }
): { x: number; y: number } {
  const [a, b, c, d, e, f] = matrix;
  return {
    x: a * point.x + c * point.y + e,
    y: b * point.x + d * point.y + f,
  };
}
