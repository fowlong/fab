import { fabric } from 'fabric';
import { fabricDeltaToPdfDelta, type Matrix } from './coords';
import type { IrDocument, IrPage, IrObject, PatchOperation } from './types';
import { mapIrObjectToFabric, type OverlayDescriptor } from './mapping';

export interface OverlayContext {
  canvas: fabric.Canvas;
  overlays: OverlayDescriptor[];
}

export function createOverlay(canvasEl: HTMLCanvasElement): OverlayContext {
  const overlays: OverlayDescriptor[] = [];
  const canvas = new fabric.Canvas(canvasEl, {
    selection: false,
    backgroundColor: 'rgba(0,0,0,0)'
  });

  const context: OverlayContext = {
    canvas,
    overlays
  };

  canvas.on('object:modified', (event) => {
    const obj = event.target as fabric.Object | undefined;
    if (!obj) return;
    const descriptor = context.overlays.find((item) => item.fabricObject === obj);
    if (!descriptor) return;
    const fold = (obj as any).__lastMatrix as Matrix | undefined;
    const fnew = obj.calcTransformMatrix() as Matrix;
    if (!fold) {
      (obj as any).__lastMatrix = fnew;
      return;
    }
    const delta = fabricDeltaToPdfDelta(fold, fnew, descriptor.pageHeightPt);
    console.log('delta matrix', delta);
    (obj as any).__lastMatrix = fnew;
  });

  return context;
}

export function syncOverlay(
  context: OverlayContext,
  page: IrPage,
  objects: IrObject[]
) {
  context.canvas.clear();
  context.overlays.length = 0;
  context.canvas.setWidth(page.widthPt);
  context.canvas.setHeight(page.heightPt);
  objects.forEach((object) => {
    const descriptor = mapIrObjectToFabric(context.canvas, page, object);
    if (descriptor) {
      (descriptor.fabricObject as any).__lastMatrix = descriptor.fabricObject.calcTransformMatrix();
      context.overlays.push(descriptor);
    }
  });
  context.canvas.renderAll();
}

export function collectTransformOps(_doc: IrDocument): PatchOperation[] {
  return [];
}
