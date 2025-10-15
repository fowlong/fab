export const POINTS_PER_INCH = 72;
export const CSS_DPI = 96;
export const SCALE = POINTS_PER_INCH / CSS_DPI;

export type Matrix = [number, number, number, number, number, number];

export function multiply(a: Matrix, b: Matrix): Matrix {
  const [a0, a1, a2, a3, a4, a5] = a;
  const [b0, b1, b2, b3, b4, b5] = b;
  return [
    a0 * b0 + a2 * b1,
    a1 * b0 + a3 * b1,
    a0 * b2 + a2 * b3,
    a1 * b2 + a3 * b3,
    a0 * b4 + a2 * b5 + a4,
    a1 * b4 + a3 * b5 + a5
  ];
}

export function invert(m: Matrix): Matrix {
  const [a, b, c, d, e, f] = m;
  const det = a * d - b * c;
  if (Math.abs(det) < Number.EPSILON) {
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
  return [SCALE, 0, 0, -SCALE, 0, pageHeightPt];
}

export function fabricDeltaToPdfDelta(
  oldMatrix: Matrix,
  newMatrix: Matrix,
  pageHeightPt: number
): Matrix {
  const deltaFabric = multiply(newMatrix, invert(oldMatrix));
  const pxToPt = pxToPtMatrix(pageHeightPt);
  const ptToPx = invert(pxToPt);
  return multiply(multiply(pxToPt, deltaFabric), ptToPx);
}
