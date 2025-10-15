export const POINTS_PER_PIXEL = 72 / 96;

export type Matrix = [number, number, number, number, number, number];

export function multiply([a1, b1, c1, d1, e1, f1]: Matrix, [a2, b2, c2, d2, e2, f2]: Matrix): Matrix {
  return [
    a1 * a2 + c1 * b2,
    b1 * a2 + d1 * b2,
    a1 * c2 + c1 * d2,
    b1 * c2 + d1 * d2,
    a1 * e2 + c1 * f2 + e1,
    b1 * e2 + d1 * f2 + f1,
  ];
}

export function invert([a, b, c, d, e, f]: Matrix): Matrix {
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

export function identity(): Matrix {
  return [1, 0, 0, 1, 0, 0];
}

export function pxToPt(pageHeightPt: number): Matrix {
  return [POINTS_PER_PIXEL, 0, 0, -POINTS_PER_PIXEL, 0, pageHeightPt];
}

export function ptToPx(pageHeightPt: number): Matrix {
  return invert(pxToPt(pageHeightPt));
}

export function fabricDeltaToPdfDelta(fabricOld: Matrix, fabricNew: Matrix, pageHeightPt: number): Matrix {
  const deltaFabric = multiply(fabricNew, invert(fabricOld));
  const pxToPtMatrix = pxToPt(pageHeightPt);
  const ptToPxMatrix = ptToPx(pageHeightPt);
  return multiply(multiply(pxToPtMatrix, deltaFabric), ptToPxMatrix);
}
