export type Matrix = [number, number, number, number, number, number];

export const POINTS_PER_PX = 72 / 96;

export function concat(m1: Matrix, m2: Matrix): Matrix {
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

export function invert(m: Matrix): Matrix {
  const [a, b, c, d, e, f] = m;
  const det = a * d - b * c;
  if (Math.abs(det) < 1e-8) {
    throw new Error('Matrix is not invertible');
  }
  const invDet = 1 / det;
  const aInv = d * invDet;
  const bInv = -b * invDet;
  const cInv = -c * invDet;
  const dInv = a * invDet;
  const eInv = -(aInv * e + cInv * f);
  const fInv = -(bInv * e + dInv * f);
  return [aInv, bInv, cInv, dInv, eInv, fInv];
}

export function pxToPtMatrix(pageHeightPt: number): Matrix {
  return [POINTS_PER_PX, 0, 0, -POINTS_PER_PX, 0, pageHeightPt];
}

export function fabricDeltaToPdfDelta(previous: Matrix, current: Matrix, pageHeightPt: number): Matrix {
  const deltaFabric = concat(current, invert(previous));
  const pxToPt = pxToPtMatrix(pageHeightPt);
  const ptToPx = invert(pxToPt);
  return concat(concat(pxToPt, deltaFabric), ptToPx);
}

export function pageTransformToFabricMatrix(pageHeightPt: number): Matrix {
  return invert(pxToPtMatrix(pageHeightPt));
}
