import { fabric } from 'fabric';
import type { DocumentIr, PatchOperation } from './types';
import { createFabricMapping } from './mapping';

export interface FabricOverlayContext {
  canvas: fabric.Canvas;
  dispose(): void;
}

export type PatchDispatcher = (ops: PatchOperation[]) => Promise<unknown>;

export async function initialiseFabricOverlay(
  preview: { canvases: HTMLCanvasElement[] },
  ir: DocumentIr,
  dispatchPatch: PatchDispatcher
): Promise<FabricOverlayContext> {
  const canvases = preview.canvases;
  if (canvases.length !== ir.pages.length) {
    console.warn('Mismatch between preview canvases and IR pages');
  }

  const overlayCanvas = document.createElement('canvas');
  overlayCanvas.width = canvases[0]?.width ?? 0;
  overlayCanvas.height = canvases[0]?.height ?? 0;
  overlayCanvas.className = 'fabric-overlay';
  canvases[0]?.parentElement?.appendChild(overlayCanvas);

  const canvas = new fabric.Canvas(overlayCanvas, {
    selection: true,
    preserveObjectStacking: true
  });

  const mapping = createFabricMapping(canvas, ir);
  mapping.initialise();

  canvas.on('object:modified', async (event) => {
    const target = event.target as fabric.Object | undefined;
    if (!target) {
      return;
    }
    const ops = mapping.createTransformPatch(target);
    if (ops.length > 0) {
      await dispatchPatch(ops);
    }
  });

  return {
    canvas,
    dispose() {
      canvas.dispose();
      overlayCanvas.remove();
    }
  };
}
