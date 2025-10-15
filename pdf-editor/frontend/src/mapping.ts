import type { fabric } from 'fabric';
import { bboxToMatrix, multiplyMatrix, ptToPxMatrix } from './coords';
import type { FabricMeta, PageIR, PageObject, TextObject } from './types';

export type FabricWithMeta = fabric.Object & { __meta?: FabricMeta };

export function attachMeta(obj: fabric.Object, meta: FabricMeta) {
  (obj as FabricWithMeta).__meta = meta;
}

export function getMeta(obj: fabric.Object): FabricMeta | undefined {
  return (obj as FabricWithMeta).__meta;
}

export function pageObjectToFabric(
  canvas: fabric.Canvas,
  page: PageIR,
  object: PageObject,
  FabricCtor: typeof fabric,
): fabric.Object {
  const { matrixPx, widthPx, heightPx } = bboxToMatrix(object.bbox, page.heightPt);
  const rect = new FabricCtor.Rect({
    width: widthPx,
    height: heightPx,
    left: 0,
    top: 0,
    fill: undefined,
    stroke: '#2563eb',
    strokeWidth: 1,
    strokeDashArray: [6, 6],
    transparentCorners: false,
    cornerColor: '#1d4ed8',
    lockScalingFlip: true,
    objectCaching: false,
    perPixelTargetFind: false,
  });

  rect.set({
    transformMatrix: matrixPx,
  });

  attachMeta(rect, {
    id: object.id,
    page: page.index,
    baseMatrixPx: matrixPx,
  });

  canvas.add(rect);
  return rect;
}

export function updateBaseMatrix(obj: fabric.Object, matrix: number[]) {
  const meta = getMeta(obj);
  if (!meta) return;
  meta.baseMatrixPx = matrix as FabricMeta['baseMatrixPx'];
}

export function fabricMatrix(obj: fabric.Object): [number, number, number, number, number, number] {
  const current = obj.calcTransformMatrix();
  return [current[0], current[1], current[2], current[3], current[4], current[5]];
}

export function textBaselineMatrix(text: TextObject, page: PageIR) {
  const ptToPx = ptToPxMatrix(page.heightPt);
  return multiplyMatrix(ptToPx, text.Tm);
}
