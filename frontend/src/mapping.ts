import { Canvas, FabricObject, Rect } from 'fabric';
import type { PageObject, PageIR } from './types';
import { ptToPxMatrix, SCALE, type Matrix } from './coords';

export type OverlayMeta = {
  id: string;
  pageIndex: number;
  baseMatrix: Matrix;
};

export function toFabricMatrix(page: PageIR, matrixPt: Matrix): Matrix {
  const ptToPx = ptToPxMatrix(page.heightPt);
  return multiply(ptToPx, matrixPt);
}

function multiply(a: Matrix, b: Matrix): Matrix {
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

export function bboxPtToPx(page: PageIR, bbox: [number, number, number, number]) {
  const [x0, y0, x1, y1] = bbox;
  const widthPt = x1 - x0;
  const heightPt = y1 - y0;
  return [
    x0 / SCALE,
    (page.heightPt - y1) / SCALE,
    widthPt / SCALE,
    heightPt / SCALE,
  ] as const;
}

export function createFabricPlaceholder(
  canvas: Canvas,
  page: PageIR,
  obj: PageObject,
): FabricObject {
  const [left, top, width, height] = bboxPtToPx(page, obj.bbox);
  const rect = new Rect({
    left,
    top,
    width,
    height,
    fill: 'rgba(0,0,0,0)',
    stroke: '#1d4ed8',
    strokeDashArray: [6, 4],
    strokeWidth: 1,
    selectable: true,
  });
  rect.set('data', {
    id: obj.id,
    pageIndex: page.index,
  } satisfies Omit<OverlayMeta, 'baseMatrix'>);
  return rect;
}
