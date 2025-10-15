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

export function inverse([a, b, c, d, e, f]: Matrix): Matrix {
  const det = a * d - b * c;
  if (Math.abs(det) < 1e-8) {
    throw new Error('Non-invertible matrix');
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

export function pxToPt(pageHeightPt: number): Matrix {
  return [POINTS_PER_PIXEL, 0, 0, -POINTS_PER_PIXEL, 0, pageHeightPt];
}

export function ptToPx(pageHeightPt: number): Matrix {
  return inverse(pxToPt(pageHeightPt));
}

export function fabricDeltaToPdfDelta(oldMatrix: Matrix, newMatrix: Matrix, pageHeightPt: number): Matrix {
  const deltaFabric = multiply(newMatrix, inverse(oldMatrix));
  const pxToPtMatrix = pxToPt(pageHeightPt);
  const ptToPxMatrix = ptToPx(pageHeightPt);
  return multiply(multiply(pxToPtMatrix, deltaFabric), ptToPxMatrix);
}
