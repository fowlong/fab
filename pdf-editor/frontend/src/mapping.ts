import { fabric } from 'fabric';
import { concat, ptToPxMatrix } from './coords';
import type { Matrix, PageObject } from './types';

export interface FabricControllerMeta {
  id: string;
  pageIndex: number;
  basePdfMatrix: Matrix;
  kind: PageObject['kind'];
}

export interface FabricController {
  object: fabric.Rect;
  meta: FabricControllerMeta;
}

export function createController(
  pageIndex: number,
  pageHeightPt: number,
  obj: PageObject
): FabricController {
  switch (obj.kind) {
    case 'text':
      return buildRectController(pageIndex, pageHeightPt, obj.id, obj.Tm, obj.bbox, obj.kind);
    case 'image':
      return buildRectController(pageIndex, pageHeightPt, obj.id, obj.cm, obj.bbox, obj.kind);
    case 'path':
      return buildRectController(pageIndex, pageHeightPt, obj.id, obj.cm, obj.bbox, obj.kind);
    default:
      throw new Error(`Unsupported object kind: ${(obj as PageObject).kind}`);
  }
}

function buildRectController(
  pageIndex: number,
  pageHeightPt: number,
  id: string,
  pdfMatrix: Matrix,
  bboxPt: [number, number, number, number],
  kind: PageObject['kind']
): FabricController {
  const [leftPt, bottomPt, rightPt, topPt] = bboxPt;
  const widthPx = (rightPt - leftPt) / 0.75;
  const heightPx = (topPt - bottomPt) / 0.75;
  const rect = new fabric.Rect({
    width: widthPx,
    height: heightPx,
    left: 0,
    top: 0,
    fill: 'rgba(0,0,0,0)',
    stroke: '#3B82F6',
    strokeWidth: 1,
    strokeDashArray: [4, 4],
    selectable: true,
    objectCaching: false,
    hasRotatingPoint: true
  });

  const ptToPx = ptToPxMatrix(pageHeightPt);
  const fabricMatrix = concat(ptToPx, pdfMatrix);
  rect.set({ transformMatrix: fabricMatrix as unknown as number[] });

  return {
    object: rect,
    meta: {
      id,
      pageIndex,
      basePdfMatrix: pdfMatrix,
      kind
    }
  };
}
