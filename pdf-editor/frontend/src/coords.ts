export type Matrix = [number, number, number, number, number, number];

export const S = 72 / 96;

export function normalizeMatrix(matrix: number[] | Matrix): Matrix {
  if (matrix.length !== 6) {
    throw new Error('Expected 6 element matrix');
  }
  return [
    matrix[0] ?? 0,
    matrix[1] ?? 0,
    matrix[2] ?? 0,
    matrix[3] ?? 0,
    matrix[4] ?? 0,
    matrix[5] ?? 0,
  ];
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
    b1 * e2 + d1 * f2 + f1,
  ];
}

export function invert(m: Matrix): Matrix {
  const [a, b, c, d, e, f] = m;
  const det = a * d - b * c;
  if (Math.abs(det) < 1e-8) {
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
  oldMatrix: number[] | Matrix,
  newMatrix: number[] | Matrix,
  pageHeightPt: number,
): Matrix {
  const normalizedOld = normalizeMatrix(oldMatrix);
  const normalizedNew = normalizeMatrix(newMatrix);
  const deltaFabric = concat(normalizedNew, invert(normalizedOld));
  const pxToPt = pxToPtMatrix(pageHeightPt);
  const ptToPx = invert(pxToPt);
  return concat(concat(pxToPt, deltaFabric), ptToPx);
}
