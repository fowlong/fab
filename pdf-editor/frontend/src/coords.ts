export const POINTS_PER_PIXEL = 72 / 96;

export type Matrix = [number, number, number, number, number, number];

export function pxToPtMatrix(pageHeightPt: number): Matrix {
  return [POINTS_PER_PIXEL, 0, 0, -POINTS_PER_PIXEL, 0, pageHeightPt];
}

export function invert(matrix: Matrix): Matrix {
  const [a, b, c, d, e, f] = matrix;
  const det = a * d - b * c;
  if (!det) {
    throw new Error('matrix not invertible');
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

export function concat(m1: Matrix, m2: Matrix): Matrix {
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

export function fabricDeltaToPdfDelta(
  fold: Matrix,
  fnew: Matrix,
  pageHeightPt: number,
): Matrix {
  const deltaFabric = concat(fnew, invert(fold));
  const pxToPt = pxToPtMatrix(pageHeightPt);
  const ptToPx = invert(pxToPt);
  return concat(concat(pxToPt, deltaFabric), ptToPx);
}

