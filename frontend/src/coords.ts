export type Matrix = [number, number, number, number, number, number];

export const POINTS_PER_PIXEL = 72 / 96;

export function multiply(m1: Matrix, m2: Matrix): Matrix {
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

export function invert(matrix: Matrix): Matrix {
  const [a, b, c, d, e, f] = matrix;
  const det = a * d - b * c;
  if (Math.abs(det) < 1e-8) {
    throw new Error("Matrix is not invertible");
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

export function fabricDeltaToPdfDelta(Fold: number[], Fnew: number[], pageHeightPt: number): Matrix {
  const deltaF = multiply(toMatrix(Fnew), invert(toMatrix(Fold)));
  const pxToPt: Matrix = [POINTS_PER_PIXEL, 0, 0, -POINTS_PER_PIXEL, 0, pageHeightPt];
  const ptToPx = invert(pxToPt);
  return multiply(multiply(pxToPt, deltaF), ptToPx);
}

export function toMatrix(values: number[] | Matrix): Matrix {
  if (values.length !== 6) {
    throw new Error("Expected 6-element matrix");
  }
  return [values[0], values[1], values[2], values[3], values[4], values[5]];
}

export function toFabricMatrix(values: number[] | Matrix): Matrix {
  return toMatrix(values);
}
