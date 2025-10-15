import type { FabricObject } from 'fabric';
import { fabric } from 'fabric';
import type { IrObject, PageIR } from './types';
import { Matrix, ptToPxMatrix } from './coords';

export type OverlayObjectMeta = {
  id: string;
  pageIndex: number;
  baseMatrix: Matrix;
};

export function createOverlayObject(
  page: PageIR,
  object: IrObject
): { fabricObject: FabricObject; meta: OverlayObjectMeta } {
  const matrixPt = object.kind === 'text' ? object.Tm : object.cm ?? [1, 0, 0, 1, 0, 0];
  const ptToPx = ptToPxMatrix(page.heightPt);
  const fabricMatrix = fabric.util.multiplyTransformMatrices(ptToPx, matrixPt) as Matrix;

  const [x0, y0, x1, y1] = object.bbox;
  const width = x1 - x0;
  const height = y1 - y0;

  const rect = new fabric.Rect({
    left: 0,
    top: 0,
    width,
    height,
    fill: 'rgba(0,0,0,0)',
    stroke: object.kind === 'text' ? '#00aaff' : '#ff7f0e',
    strokeWidth: 1,
    originX: 'left',
    originY: 'top',
    transparentCorners: false
  });

  rect.set('matrix', fabricMatrix);

  const meta: OverlayObjectMeta = {
    id: object.id,
    pageIndex: page.index,
    baseMatrix: matrixPt
  };

  return { fabricObject: rect, meta };
}

export function applyMetaToObject(target: FabricObject, meta: OverlayObjectMeta) {
  target.set('data', meta);
}

export function extractMeta(target: FabricObject): OverlayObjectMeta | undefined {
  return target.get('data') as OverlayObjectMeta | undefined;
}
