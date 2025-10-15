import type { Matrix } from "./types";

export const POINTS_PER_INCH = 72;
export const CSS_DPI = 96;
export const SCALE = POINTS_PER_INCH / CSS_DPI;

export function pxToPtMatrix(pageHeightPt: number): Matrix {
  return [SCALE, 0, 0, -SCALE, 0, pageHeightPt];
}

export function multiplyMatrix(a: Matrix, b: Matrix): Matrix {
  const [a1, b1, c1, d1, e1, f1] = a;
  const [a2, b2, c2, d2, e2, f2] = b;
  return [
    a1 * a2 + b1 * c2,
    a1 * b2 + b1 * d2,
    c1 * a2 + d1 * c2,
    c1 * b2 + d1 * d2,
    e1 * a2 + f1 * c2 + e2,
    e1 * b2 + f1 * d2 + f2,
  ];
}

export function invertMatrix(m: Matrix): Matrix {
  const [a, b, c, d, e, f] = m;
  const det = a * d - b * c;
  if (Math.abs(det) < Number.EPSILON) {
    throw new Error("Matrix not invertible");
  }
  const invDet = 1 / det;
  const aInv = d * invDet;
  const bInv = -b * invDet;
  const cInv = -c * invDet;
  const dInv = a * invDet;
  const eInv = -(aInv * e + cInv * f);
  const fInv = -(bInv * e + dInv * f);
  return [aInv, bInv, cInv, dInv, eInv, fInv];
}

export function fabricDeltaToPdfDelta(
  oldMatrix: Matrix,
  newMatrix: Matrix,
  pageHeightPt: number
): Matrix {
  const deltaFabric = multiplyMatrix(newMatrix, invertMatrix(oldMatrix));
  const pxToPt = pxToPtMatrix(pageHeightPt);
  const ptToPx = invertMatrix(pxToPt);
  return multiplyMatrix(multiplyMatrix(pxToPt, deltaFabric), ptToPx);
}
