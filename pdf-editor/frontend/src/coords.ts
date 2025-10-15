export const POINTS_PER_PIXEL = 72 / 96;

export type Matrix = [number, number, number, number, number, number];

export function multiply(a: Matrix, b: Matrix): Matrix {
  const [a0, a1, a2, a3, a4, a5] = a;
  const [b0, b1, b2, b3, b4, b5] = b;
  return [
    a0 * b0 + a2 * b1,
    a1 * b0 + a3 * b1,
    a0 * b2 + a2 * b3,
    a1 * b2 + a3 * b3,
    a0 * b4 + a2 * b5 + a4,
    a1 * b4 + a3 * b5 + a5,
  ];
}

export function invert(m: Matrix): Matrix {
  const [a, b, c, d, e, f] = m;
  const det = a * d - b * c;
  if (Math.abs(det) < 1e-8) {
    throw new Error("Matrix not invertible");
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

export function ptToPxMatrix(pageHeightPt: number): Matrix {
  return invert(pxToPtMatrix(pageHeightPt));
}

export function fabricDeltaToPdfDelta(
  oldMatrix: number[],
  newMatrix: number[],
  pageHeightPt: number,
): Matrix {
  const ΔF = multiply(newMatrix as Matrix, invert(oldMatrix as Matrix));
  const PxToPt = pxToPtMatrix(pageHeightPt);
  const PtToPx = invert(PxToPt);
  return multiply(multiply(PxToPt, ΔF), PtToPx);
}
