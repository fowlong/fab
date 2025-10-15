import { fabric } from 'fabric';
import type { IrObject, IrPage } from './types';
import { pxToPtMatrix, invert, multiply, type Matrix } from './coords';

export interface OverlayDescriptor {
  fabricObject: fabric.Object;
  source: IrObject;
  initialMatrix: Matrix;
  pageHeightPt: number;
}

export function mapIrObjectToFabric(
  canvas: fabric.Canvas,
  page: IrPage,
  object: IrObject
): OverlayDescriptor | null {
  const [x1, y1, x2, y2] = object.bbox;
  const width = x2 - x1;
  const height = y2 - y1;
  const rect = new fabric.Rect({
    left: x1,
    top: page.heightPt - y2,
    width,
    height,
    fill: 'rgba(56, 189, 248, 0.1)',
    stroke: 'rgba(56, 189, 248, 0.8)',
    strokeDashArray: [6, 4],
    selectable: true,
    hasBorders: true,
    hasControls: true,
    hoverCursor: 'move'
  });

  const pageMatrixPt = pxToPtMatrix(page.heightPt);
  const identityPx: Matrix = [1, 0, 0, 1, 0, 0];
  const ptToPx = invert(pageMatrixPt);
  const initialMatrix = multiply(pageMatrixPt, identityPx);
  const descriptor: OverlayDescriptor = {
    fabricObject: rect,
    source: object,
    initialMatrix: multiply(initialMatrix, ptToPx),
    pageHeightPt: page.heightPt
  };

  canvas.add(rect);
  return descriptor;
}
