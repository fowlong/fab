export const POINTS_PER_INCH = 72;
export const CSS_DPI = 96;
export const SCALE = POINTS_PER_INCH / CSS_DPI; // 0.75

export type Matrix = [number, number, number, number, number, number];

export function multiply(a: Matrix, b: Matrix): Matrix {
  const [a1, a2, a3, a4, a5, a6] = a;
  const [b1, b2, b3, b4, b5, b6] = b;
  return [
    a1 * b1 + a3 * b2,
    a2 * b1 + a4 * b2,
    a1 * b3 + a3 * b4,
    a2 * b3 + a4 * b4,
    a1 * b5 + a3 * b6 + a5,
    a2 * b5 + a4 * b6 + a6
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

export function pxToPtMatrix(pageHeightPt: number): Matrix {
  return [SCALE, 0, 0, -SCALE, 0, pageHeightPt];
}

export function ptToPxMatrix(pageHeightPt: number): Matrix {
  return invert(pxToPtMatrix(pageHeightPt));
}

export function fabricDeltaToPdfDelta(
  oldMatrix: Matrix,
  newMatrix: Matrix,
  pageHeightPt: number
): Matrix {
  const deltaFabric = multiply(newMatrix, invert(oldMatrix));
  const pxToPt = pxToPtMatrix(pageHeightPt);
  const ptToPx = invert(pxToPt);
  return multiply(multiply(pxToPt, deltaFabric), ptToPx);
}
