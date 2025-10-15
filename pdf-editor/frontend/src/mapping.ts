import type { IrObject, PageIR } from './types';
import type { Matrix } from './coords';
import { pxToPt, ptToPx } from './coords';

export interface FabricMapping {
  id: string;
  pageIndex: number;
  initialMatrixPx: Matrix;
  bboxPx: [number, number, number, number];
  object: IrObject;
}

function applyMatrixToPoint([a, b, c, d, e, f]: Matrix, x: number, y: number): [number, number] {
  return [a * x + c * y + e, b * x + d * y + f];
}

export function mapObjectToFabric(page: PageIR, object: IrObject): FabricMapping {
  const toPx = ptToPx(page.heightPt);
  const [x0, y0] = applyMatrixToPoint(toPx, object.bbox[0], object.bbox[1]);
  const [x1, y1] = applyMatrixToPoint(toPx, object.bbox[2], object.bbox[3]);
  const bboxPx: [number, number, number, number] = [x0, y0, x1, y1];
  const initialMatrixPx: Matrix = toPx;
  return {
    id: object.id,
    pageIndex: page.index,
    initialMatrixPx,
    bboxPx,
    object,
  };
}

export function mapPageObjects(pages: PageIR[]): FabricMapping[] {
  return pages.flatMap((page) => page.objects.map((object) => mapObjectToFabric(page, object)));
}

export function fabricMatrixToPdfMatrix(matrixPx: Matrix, pageHeightPt: number): Matrix {
  const toPt = pxToPt(pageHeightPt);
  return [
    toPt[0] * matrixPx[0] + toPt[2] * matrixPx[1],
    toPt[1] * matrixPx[0] + toPt[3] * matrixPx[1],
    toPt[0] * matrixPx[2] + toPt[2] * matrixPx[3],
    toPt[1] * matrixPx[2] + toPt[3] * matrixPx[3],
    toPt[0] * matrixPx[4] + toPt[2] * matrixPx[5] + toPt[4],
    toPt[1] * matrixPx[4] + toPt[3] * matrixPx[5] + toPt[5],
  ];
}
