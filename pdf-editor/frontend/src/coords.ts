import type { Matrix } from './types';

export const CSS_DPI = 96;
export const PDF_DPI = 72;
export const S = PDF_DPI / CSS_DPI;

export function pxToPtMatrix(pageHeightPt: number): Matrix {
  return [S, 0, 0, -S, 0, pageHeightPt];
}

export function ptToPxMatrix(pageHeightPt: number): Matrix {
  return invert(pxToPtMatrix(pageHeightPt));
}

export function identity(): Matrix {
  return [1, 0, 0, 1, 0, 0];
}

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

export function invert(m: Matrix): Matrix {
  const [a, b, c, d, e, f] = m;
  const det = a * d - b * c;
  if (Math.abs(det) < 1e-8) {
    throw new Error('Matrix not invertible');
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

export function fabricDeltaToPdfDelta(
  fold: Matrix,
  fnew: Matrix,
  pageHeightPt: number
): Matrix {
  const deltaFabric = concat(fnew, invert(fold));
  const pxToPt = pxToPtMatrix(pageHeightPt);
  const ptToPx = invert(pxToPt);
  return concat(concat(pxToPt, deltaFabric), ptToPx);
}

export function bboxPxToPt(
  bboxPx: [number, number, number, number],
  pageHeightPt: number
): [number, number, number, number] {
  const [x1, y1, x2, y2] = bboxPx;
  const mat = pxToPtMatrix(pageHeightPt);
  const [x1p, y1p] = applyToPoint(mat, x1, y1);
  const [x2p, y2p] = applyToPoint(mat, x2, y2);
  const left = Math.min(x1p, x2p);
  const right = Math.max(x1p, x2p);
  const bottom = Math.min(y1p, y2p);
  const top = Math.max(y1p, y2p);
  return [left, bottom, right, top];
}

export function applyToPoint(matrix: Matrix, x: number, y: number): [number, number] {
  const [a, b, c, d, e, f] = matrix;
  return [a * x + c * y + e, b * x + d * y + f];
}
