export const POINTS_PER_INCH = 72;
export const CSS_PX_PER_INCH = 96;
export const SCALE = POINTS_PER_INCH / CSS_PX_PER_INCH;

type Matrix = [number, number, number, number, number, number];

export function pxToPtMatrix(pageHeightPt: number): Matrix {
  return [SCALE, 0, 0, -SCALE, 0, pageHeightPt];
}

export function multiplyMatrices(m1: Matrix, m2: Matrix): Matrix {
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

export function invertMatrix([a, b, c, d, e, f]: Matrix): Matrix {
  const det = a * d - b * c;
  if (Math.abs(det) < Number.EPSILON) {
    throw new Error('Matrix is not invertible');
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
  fabricOld: Matrix,
  fabricNew: Matrix,
  pageHeightPt: number
): Matrix {
  const deltaFabric = multiplyMatrices(fabricNew, invertMatrix(fabricOld));
  const pxToPt = pxToPtMatrix(pageHeightPt);
  const ptToPx = invertMatrix(pxToPt);
  return multiplyMatrices(multiplyMatrices(pxToPt, deltaFabric), ptToPx);
}
