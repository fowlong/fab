export type Matrix = [number, number, number, number, number, number];

export const S = 72 / 96; // px → pt scale factor

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

export function pxToPtMatrix(pageHeightPt: number): Matrix {
  return [S, 0, 0, -S, 0, pageHeightPt];
}

export function fabricDeltaToPdfDelta(fold: Matrix, fnew: Matrix, pageHeightPt: number): Matrix {
  const deltaFabric = multiply(fnew, invert(fold));
  const pxToPt = pxToPtMatrix(pageHeightPt);
  const ptToPx = invert(pxToPt);
  return multiply(multiply(pxToPt, deltaFabric), ptToPx);
}

export function ptBboxToPx(
  pageHeightPt: number,
  bbox: [number, number, number, number],
): readonly [number, number, number, number] {
  const [x0, y0, x1, y1] = bbox;
  const widthPt = x1 - x0;
  const heightPt = y1 - y0;
  return [x0 / S, (pageHeightPt - y1) / S, widthPt / S, heightPt / S] as const;
}
