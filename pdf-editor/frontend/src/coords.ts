export const POINTS_PER_PIXEL = 72 / 96;

export type Matrix = [number, number, number, number, number, number];

export function multiply(m1: Matrix, m2: Matrix): Matrix {
  const [a1, b1, c1, d1, e1, f1] = m1;
  const [a2, b2, c2, d2, e2, f2] = m2;
  const a = a1 * a2 + b1 * c2;
  const b = a1 * b2 + b1 * d2;
  const c = c1 * a2 + d1 * c2;
  const d = c1 * b2 + d1 * d2;
  const e = e1 * a2 + f1 * c2 + e2;
  const f = e1 * b2 + f1 * d2 + f2;
  return [a, b, c, d, e, f];
}

export function invert(m: Matrix): Matrix {
  const [a, b, c, d, e, f] = m;
  const det = a * d - b * c;
  if (!det) {
    return [1, 0, 0, 1, 0, 0];
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

export function fabricDeltaToPdfDelta(
  fold: Matrix,
  fnew: Matrix,
  pageHeightPt: number
): Matrix {
  const deltaF = multiply(fnew, invert(fold));
  const pxToPt = pxToPtMatrix(pageHeightPt);
  const ptToPx = invert(pxToPt);
  return multiply(multiply(pxToPt, deltaF), ptToPx);
}
