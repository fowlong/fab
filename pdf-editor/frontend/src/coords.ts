export const POINTS_PER_INCH = 72;
export const CSS_PX_PER_INCH = 96;
export const S = POINTS_PER_INCH / CSS_PX_PER_INCH;

export type Matrix = [number, number, number, number, number, number];

export function multiplyMatrix(a: Matrix, b: Matrix): Matrix {
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

export function invertMatrix([a, b, c, d, e, f]: Matrix): Matrix {
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
  return [S, 0, 0, -S, 0, pageHeightPt];
}

export function ptToPxMatrix(pageHeightPt: number): Matrix {
  return invertMatrix(pxToPtMatrix(pageHeightPt));
}

export function fabricDeltaToPdfDelta(
  previous: Matrix,
  next: Matrix,
  pageHeightPt: number,
): Matrix {
  const deltaFabric = multiplyMatrix(next, invertMatrix(previous));
  const pxToPt = pxToPtMatrix(pageHeightPt);
  const ptToPx = ptToPxMatrix(pageHeightPt);
  return multiplyMatrix(multiplyMatrix(pxToPt, deltaFabric), ptToPx);
}

export function applyMatrixToPoint(matrix: Matrix, x: number, y: number) {
  const [a, b, c, d, e, f] = matrix;
  return {
    x: a * x + c * y + e,
    y: b * x + d * y + f,
  };
}

export function bboxToMatrix(
  bbox: [number, number, number, number],
  pageHeightPt: number,
): { matrixPx: Matrix; widthPx: number; heightPx: number } {
  const [x0, y0, x1, y1] = bbox;
  const widthPt = x1 - x0;
  const heightPt = y1 - y0;
  const ptToPx = ptToPxMatrix(pageHeightPt);
  const translation = multiplyMatrix(ptToPx, [1, 0, 0, 1, x0, y0]);
  return {
    matrixPx: translation,
    widthPx: widthPt / S,
    heightPx: heightPt / S,
  };
}
