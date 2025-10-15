import type { Matrix } from './types';

export const S = 72 / 96;

export function multiply(m1: Matrix, m2: Matrix): Matrix {
  const [a1, b1, c1, d1, e1, f1] = m1;
  const [a2, b2, c2, d2, e2, f2] = m2;
  return [
    a1 * a2 + b1 * c2,
    a1 * b2 + b1 * d2,
    c1 * a2 + d1 * c2,
    c1 * b2 + d1 * d2,
    e1 * a2 + f1 * c2 + e2,
    e1 * b2 + f1 * d2 + f2,
  ];
}

export function invert(m: Matrix): Matrix {
  const [a, b, c, d, e, f] = m;
  const det = a * d - b * c;
  if (Math.abs(det) < Number.EPSILON) {
    throw new Error('Matrix not invertible');
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
  return [S, 0, 0, -S, 0, pageHeightPt];
}

export function fabricDeltaToPdfDelta(
  original: Matrix,
  next: Matrix,
  pageHeightPt: number,
): Matrix {
  const deltaFabric = multiply(next, invert(original));
  const pxToPt = pxToPtMatrix(pageHeightPt);
  const ptToPx = invert(pxToPt);
  return multiply(multiply(pxToPt, deltaFabric), ptToPx);
}
