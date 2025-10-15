import type { Matrix } from './coords';
import { pxToPtMatrix, ptToPxMatrix, multiply } from './coords';
import type { PageObject } from './types';

export type OverlayObjectMeta = {
  id: string;
  pageIndex: number;
  pdfMatrix: Matrix;
};

export function objectBoundingMatrix(
  object: PageObject,
  pageHeightPt: number,
): Matrix {
  const ptToPx = ptToPxMatrix(pageHeightPt);
  const pdfMatrix: Matrix =
    object.kind === 'text'
      ? object.Tm
      : object.kind === 'image'
        ? object.cm
        : object.cm;
  return multiply(ptToPx, pdfMatrix);
}

export function bboxToFabricRect(
  bbox: [number, number, number, number],
  pageHeightPt: number,
) {
  const [x1, y1, x2, y2] = bbox;
  const ptToPx = ptToPxMatrix(pageHeightPt);
  const topLeft = apply(ptToPx, x1, y2);
  const bottomRight = apply(ptToPx, x2, y1);
  return {
    left: topLeft[0],
    top: topLeft[1],
    width: bottomRight[0] - topLeft[0],
    height: bottomRight[1] - topLeft[1],
  };
}

function apply(m: Matrix, x: number, y: number): [number, number] {
  const [a, b, c, d, e, f] = m;
  return [a * x + c * y + e, b * x + d * y + f];
}

export function pdfMatrixToFabric(
  matrix: Matrix,
  pageHeightPt: number,
): Matrix {
  const ptToPx = ptToPxMatrix(pageHeightPt);
  const pxToPt = pxToPtMatrix(pageHeightPt);
  return multiply(ptToPx, multiply(matrix, pxToPt));
}
