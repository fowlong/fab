export const POINTS_PER_INCH = 72;
export const CSS_PX_PER_INCH = 96;
export const SCALE = POINTS_PER_INCH / CSS_PX_PER_INCH;

export type Matrix = [number, number, number, number, number, number];

export function multiplyMatrix(a: Matrix, b: Matrix): Matrix {
  const [a0, a1, a2, a3, a4, a5] = a;
  const [b0, b1, b2, b3, b4, b5] = b;
  return [
    a0 * b0 + a2 * b1,
    a1 * b0 + a3 * b1,
    a0 * b2 + a2 * b3,
    a1 * b2 + a3 * b3,
    a0 * b4 + a2 * b5 + a4,
    a1 * b4 + a3 * b5 + a5,
  ];
}

export function determinant(m: Matrix): number {
  return m[0] * m[3] - m[1] * m[2];
}

export function invertMatrix(m: Matrix): Matrix {
  const det = determinant(m);
  if (Math.abs(det) < 1e-8) {
    throw new Error('Matrix is not invertible');
  }
  const [a, b, c, d, e, f] = m;
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
  return [SCALE, 0, 0, -SCALE, 0, pageHeightPt];
}

export function ptToPxMatrix(pageHeightPt: number): Matrix {
  return invertMatrix(pxToPtMatrix(pageHeightPt));
}

export function fabricDeltaToPdfDelta(
  prev: Matrix,
  next: Matrix,
  pageHeightPt: number,
): Matrix {
  const deltaFabric = multiplyMatrix(next, invertMatrix(prev));
  const pxToPt = pxToPtMatrix(pageHeightPt);
  const ptToPx = invertMatrix(pxToPt);
  return multiplyMatrix(multiplyMatrix(pxToPt, deltaFabric), ptToPx);
}
