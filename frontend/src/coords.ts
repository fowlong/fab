export const S = 72 / 96;

export type Matrix = [number, number, number, number, number, number];

export function pxToPtMatrix(pageHeightPt: number): Matrix {
  return [S, 0, 0, -S, 0, pageHeightPt];
}

export function invert(matrix: Matrix): Matrix {
  const [a, b, c, d, e, f] = matrix;
  const det = a * d - b * c;
  if (Math.abs(det) < 1e-8) {
    throw new Error('Matrix is not invertible');
  }
  const invDet = 1 / det;
  const a1 = d * invDet;
  const b1 = -b * invDet;
  const c1 = -c * invDet;
  const d1 = a * invDet;
  const e1 = (c * f - d * e) * invDet;
  const f1 = (b * e - a * f) * invDet;
  return [a1, b1, c1, d1, e1, f1];
}

export function multiply(a: Matrix, b: Matrix): Matrix {
  return [
    a[0] * b[0] + a[2] * b[1],
    a[1] * b[0] + a[3] * b[1],
    a[0] * b[2] + a[2] * b[3],
    a[1] * b[2] + a[3] * b[3],
    a[0] * b[4] + a[2] * b[5] + a[4],
    a[1] * b[4] + a[3] * b[5] + a[5],
  ];
}

export function fabricDeltaToPdfDelta(
  fold: Matrix,
  fnew: Matrix,
  pageHeightPt: number,
): Matrix {
  const deltaFabric = multiply(fnew, invert(fold));
  const pxToPt = pxToPtMatrix(pageHeightPt);
  const ptToPx = invert(pxToPt);
  return multiply(multiply(pxToPt, deltaFabric), ptToPx);
}
