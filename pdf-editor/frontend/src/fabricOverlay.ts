import { fabric } from 'fabric';
import type { Matrix } from './coords';
import type { PatchOperation } from './types';

export interface FabricObjectMeta {
  id: string;
  kind: 'text' | 'image' | 'path';
  initialMatrix: Matrix;
}

export interface OverlayController {
  canvas: fabric.Canvas;
  updateObjects(meta: FabricObjectMeta[]): void;
  dispose(): void;
}

export interface OverlayCallbacks {
  onObjectTransform?: (meta: FabricObjectMeta, newMatrix: Matrix) => void;
  onRequestEditText?: (meta: FabricObjectMeta) => void;
}

export function createOverlay(
  canvasElement: HTMLCanvasElement,
  callbacks: OverlayCallbacks
): OverlayController {
  const canvas = new fabric.Canvas(canvasElement, {
    selection: false,
    fireRightClick: true,
    preserveObjectStacking: true,
  });

  const controller: OverlayController = {
    canvas,
    updateObjects(metaList: FabricObjectMeta[]) {
      canvas.clear();
      metaList.forEach((meta) => {
        const rect = new fabric.Rect({
          left: 0,
          top: 0,
          width: 100,
          height: 100,
          fill: 'rgba(0,0,0,0)',
          stroke: '#1a73e8',
          strokeDashArray: [6, 4],
          hasBorders: true,
          hasControls: true,
        });
        rect.data = meta;
        canvas.add(rect);
      });
      canvas.renderAll();
    },
    dispose() {
      canvas.dispose();
    },
  };

  canvas.on('object:modified', (event) => {
    const obj = event.target;
    if (!obj?.data) return;
    const meta = obj.data as FabricObjectMeta;
    const matrix = (obj.calcTransformMatrix?.() ?? obj.transformMatrix) as
      | Matrix
      | undefined;
    if (!matrix) return;
    callbacks.onObjectTransform?.(meta, [
      matrix[0],
      matrix[1],
      matrix[2],
      matrix[3],
      matrix[4],
      matrix[5],
    ]);
  });

  canvas.on('mouse:dblclick', (event) => {
    const obj = event.target;
    if (!obj?.data) return;
    callbacks.onRequestEditText?.(obj.data as FabricObjectMeta);
  });

  return controller;
}

export function applyPatchToOverlay(
  controller: OverlayController,
  _patch: PatchOperation
) {
  // Placeholder implementation: in the MVP skeleton the Fabric overlay is not yet
  // applying patches locally. The server response is considered the source of
  // truth and will refresh the overlay when reloading the IR.
  void controller;
}
