import type { IrObject, PdfPreviewPage } from './types';
import type { Matrix } from './coords';
import { ptToPxMatrix } from './coords';

export interface FabricObjectMeta {
  id: string;
  pageIndex: number;
  baseMatrix: Matrix;
  ir: IrObject;
}

export function bboxToFabricMatrix(page: PdfPreviewPage, object: IrObject): Matrix {
  const ptToPx = ptToPxMatrix(pageHeight(page));
  const [x1, y1, x2, y2] = object.bbox;
  const width = x2 - x1;
  const height = y2 - y1;
  const [a, b, c, d, e, f] = ptToPx;
  const pxX = a * x1 + c * y1 + e;
  const pxY = b * x1 + d * y1 + f;
  return [1, 0, 0, 1, pxX, pxY + height];
}

function pageHeight(page: PdfPreviewPage): number {
  const canvas = page.canvas;
  const pixelHeight = canvas.height;
  const scale = canvas.dataset.scale ? Number(canvas.dataset.scale) : 1;
  const heightPx = pixelHeight / scale;
  return heightPx * (72 / 96);
}
