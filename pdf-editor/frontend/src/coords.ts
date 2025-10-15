export const S = 72 / 96;

export type Matrix = [number, number, number, number, number, number];

export function concat([a1, b1, c1, d1, e1, f1]: Matrix, [a2, b2, c2, d2, e2, f2]: Matrix): Matrix {
  return [
    a1 * a2 + c1 * b2,
    b1 * a2 + d1 * b2,
    a1 * c2 + c1 * d2,
    b1 * c2 + d1 * d2,
    a1 * e2 + c1 * f2 + e1,
    b1 * e2 + d1 * f2 + f1
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

export function pxToPtMatrix(pageHeightPt: number): Matrix {
  return [S, 0, 0, -S, 0, pageHeightPt];
}

export function ptToPxMatrix(pageHeightPt: number): Matrix {
  return invert(pxToPtMatrix(pageHeightPt));
}

export function fabricDeltaToPdfDelta(base: number[], current: number[], pageHeightPt: number): Matrix {
  const deltaFabric = concat(current as Matrix, invert(base as Matrix));
  const pxToPt = pxToPtMatrix(pageHeightPt);
  const ptToPx = invert(pxToPt);
  return concat(concat(pxToPt, deltaFabric), ptToPx);
}
