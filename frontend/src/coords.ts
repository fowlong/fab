import type { Matrix } from './types';

export const S = 72 / 96;
export const PX_PER_PT = 1 / S;

export function pxToPtMatrix(pageHeightPt: number): Matrix {
  return [S, 0, 0, -S, 0, pageHeightPt];
}

export function invert(m: Matrix): Matrix {
  const [a, b, c, d, e, f] = m;
  const det = a * d - b * c;
  if (Math.abs(det) < 1e-8) {
    throw new Error('Matrix not invertible');
  }
  const invA = d / det;
  const invB = -b / det;
  const invC = -c / det;
  const invD = a / det;
  const invE = (c * f - d * e) / det;
  const invF = (b * e - a * f) / det;
  return [invA, invB, invC, invD, invE, invF];
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

export function fabricDeltaToPdfDelta(Fold: Matrix, Fnew: Matrix, pageHeightPt: number): Matrix {
  const deltaFabric = multiply(Fnew, invert(Fold));
  const pxToPt = pxToPtMatrix(pageHeightPt);
  const ptToPx = invert(pxToPt);
  return multiply(multiply(pxToPt, deltaFabric), ptToPx);
}
