import type { PageObject } from './types';
import { multiplyMatrices, pxToPtMatrix, invertMatrix } from './coords';

type Matrix = [number, number, number, number, number, number];

export function pdfMatrixToFabric(
  matrix: Matrix,
  pageHeightPt: number
): Matrix {
  const pxToPt = pxToPtMatrix(pageHeightPt);
  const ptToPx = invertMatrix(pxToPt);
  return multiplyMatrices(multiplyMatrices(ptToPx, matrix), pxToPt);
}

export function objectBoundingBoxPx(object: PageObject, pageHeightPt: number) {
  const [x1, y1, x2, y2] = object.bbox;
  const scale = 1 / (72 / 96);
  const top = (pageHeightPt - y2) * scale;
  const left = x1 * scale;
  const width = (x2 - x1) * scale;
  const height = (y2 - y1) * scale;
  return { top, left, width, height };
}
