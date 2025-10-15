import { fabric } from 'fabric';
import type { Matrix } from './coords';
import { fabricDeltaToPdfDelta } from './coords';
import type { PageObject } from './types';
import { mapObjectToOverlay } from './mapping';

export type TransformCallback = (
  object: PageObject,
  pageIndex: number,
  deltaMatrix: Matrix,
) => void;

export type OverlayController = {
  canvas: fabric.Canvas;
  dispose(): void;
};

type OverlayMeta = {
  object: PageObject;
  baseMatrix: Matrix;
};

export function createOverlay(
  canvasElement: HTMLCanvasElement,
  pageHeightPt: number,
  pageIndex: number,
  objects: PageObject[],
  onTransform: TransformCallback,
): OverlayController {
  const canvas = new fabric.Canvas(canvasElement, {
    selection: true,
    preserveObjectStacking: true,
    backgroundColor: 'rgba(0,0,0,0)',
  });
  canvas.setWidth(canvasElement.width);
  canvas.setHeight(canvasElement.height);
  canvasElement.classList.add('fabric-overlay');

  const metas = new Map<string, OverlayMeta>();

  objects.forEach((object) => {
    const overlay = mapObjectToOverlay(pageHeightPt, object);
    const rect = new fabric.Rect({
      left: 0,
      top: 0,
      width: overlay.widthPx,
      height: overlay.heightPx,
      fill: 'rgba(59, 130, 246, 0.08)',
      stroke: '#2563eb',
      strokeWidth: 1,
      opacity: 0.9,
      selectable: true,
      evented: true,
      hasBorders: true,
      hasControls: true,
      transparentCorners: false,
      originX: 'left',
      originY: 'top',
    });
    rect.set('transformMatrix', overlay.matrixPx as unknown as number[]);
    rect.data = object.id;
    metas.set(object.id, { object, baseMatrix: overlay.matrixPx });
    canvas.add(rect);
  });

  canvas.on('object:modified', (event) => {
    const target = event.target as fabric.Object | undefined;
    if (!target) return;
    const objectId = target.data as string;
    if (!objectId) return;
    const meta = metas.get(objectId);
    if (!meta) return;
    const matrix = target.calcTransformMatrix() as unknown as Matrix;
    try {
      const delta = fabricDeltaToPdfDelta(
        meta.baseMatrix,
        matrix,
        pageHeightPt,
      );
      onTransform(meta.object, pageIndex, delta);
      metas.set(objectId, { object: meta.object, baseMatrix: matrix });
    } catch (error) {
      console.error('Failed to compute transform delta', error);
    }
  });

  const dispose = () => {
    canvas.dispose();
    metas.clear();
  };

  return { canvas, dispose };
}
