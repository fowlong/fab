export const POINTS_PER_PIXEL = 72 / 96;

export type Matrix = [number, number, number, number, number, number];

export function multiplyMatrix(a: Matrix, b: Matrix): Matrix {
  const [a1, a2, a3, a4, a5, a6] = a;
  const [b1, b2, b3, b4, b5, b6] = b;
  return [
    a1 * b1 + a3 * b2,
    a2 * b1 + a4 * b2,
    a1 * b3 + a3 * b4,
    a2 * b3 + a4 * b4,
    a1 * b5 + a3 * b6 + a5,
    a2 * b5 + a4 * b6 + a6,
  ];
}

export function invertMatrix(m: Matrix): Matrix {
  const [a, b, c, d, e, f] = m;
  const det = a * d - b * c;
  if (Math.abs(det) < 1e-10) {
    throw new Error("Matrix not invertible");
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

export function pixelToPdfMatrix(pageHeightPt: number): Matrix {
  return [POINTS_PER_PIXEL, 0, 0, -POINTS_PER_PIXEL, 0, pageHeightPt];
}

export function fabricDeltaToPdfDelta(oldMatrix: number[], newMatrix: number[], pageHeightPt: number): Matrix {
  const deltaFabric = multiplyMatrix(newMatrix as Matrix, invertMatrix(oldMatrix as Matrix));
  const pxToPt = pixelToPdfMatrix(pageHeightPt);
  const ptToPx = invertMatrix(pxToPt);
  return multiplyMatrix(multiplyMatrix(pxToPt, deltaFabric), ptToPx);
}
