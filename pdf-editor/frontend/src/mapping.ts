import { fabric } from 'fabric';
import type { PageObject, TextObject } from './types';
import { ptToPxMatrix, multiply } from './coords';
import type { PdfMatrix } from './types';

export interface FabricControllerMeta {
  id: string;
  pageIndex: number;
  kind: PageObject['kind'];
  baseMatrix: PdfMatrix;
}

export function attachMeta<T extends fabric.Object>(
  obj: T,
  meta: FabricControllerMeta
): T {
  (obj as T & { controllerMeta: FabricControllerMeta }).controllerMeta = meta;
  return obj;
}

export function getMeta(obj: fabric.Object): FabricControllerMeta | undefined {
  return (obj as fabric.Object & { controllerMeta?: FabricControllerMeta })
    .controllerMeta;
}

export function controllerFromObject(
  pageIndex: number,
  object: PageObject,
  pageHeightPt: number
): fabric.Rect {
  const [x0, y0, x1, y1] = object.bbox;
  const widthPt = x1 - x0;
  const heightPt = y1 - y0;
  const ptToPx = ptToPxMatrix(pageHeightPt);
  const topLeftPx = transformPoint({ x: x0, y: y1 }, ptToPx);
  const widthPx = widthPt / 72 * 96;
  const heightPx = heightPt / 72 * 96;

  const rect = new fabric.Rect({
    left: topLeftPx.x,
    top: topLeftPx.y,
    width: widthPx,
    height: heightPx,
    fill: 'rgba(0,0,0,0)',
    stroke: object.kind === 'text' ? '#1f77b4' : '#ff7f0e',
    strokeWidth: 1,
    selectable: true,
    hasBorders: true,
    hasControls: true
  });

  const baseMatrix = initialMatrix(object, pageHeightPt);
  rect.transformMatrix = baseMatrix.slice() as unknown as number[];

  attachMeta(rect, {
    id: object.id,
    pageIndex,
    kind: object.kind,
    baseMatrix
  });

  return rect;
}

function transformPoint(
  point: { x: number; y: number },
  matrix: PdfMatrix
): { x: number; y: number } {
  const [a, b, c, d, e, f] = matrix;
  return {
    x: a * point.x + c * point.y + e,
    y: b * point.x + d * point.y + f
  };
}

function initialMatrix(object: PageObject, pageHeightPt: number): PdfMatrix {
  if (object.kind === 'text') {
    const text = object as TextObject;
    const textMatrix = text.Tm;
    const ptToPx = ptToPxMatrix(pageHeightPt);
    return multiply(ptToPx, textMatrix);
  }
  const cm = object.cm;
  const ptToPx = ptToPxMatrix(pageHeightPt);
  return multiply(ptToPx, cm);
}
