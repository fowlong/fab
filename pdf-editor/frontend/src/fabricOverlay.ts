import { fabric } from 'fabric';
import { fabricDeltaToPdfDelta } from './coords';
import type { Matrix, PageIR, PatchOp, TransformPatch } from './types';
import type { RenderedPage } from './pdfPreview';
import { createController, type FabricMetadata } from './mapping';

export interface OverlayCallbacks {
  onTransform: (op: PatchOp) => void;
}

export interface OverlayHandle {
  dispose(): void;
}

function matrixFromTarget(target: fabric.Object): Matrix {
  const values = target.calcTransformMatrix();
  if (values.length < 6) {
    throw new Error('Unexpected matrix size from Fabric object');
  }
  return [values[0], values[1], values[2], values[3], values[4], values[5]];
}

function updateMetadata(target: fabric.Object, matrix: Matrix) {
  const metadata = (target as any).metadata as FabricMetadata | undefined;
  if (metadata) {
    metadata.baseMatrix = matrix;
  }
}

export function setupOverlay(
  pageWrapper: HTMLElement,
  rendered: RenderedPage,
  page: PageIR,
  callbacks: OverlayCallbacks
): OverlayHandle {
  const overlayCanvas = document.createElement('canvas');
  overlayCanvas.width = rendered.widthPx;
  overlayCanvas.height = rendered.heightPx;
  overlayCanvas.className = 'fabric-overlay';
  overlayCanvas.style.position = 'absolute';
  overlayCanvas.style.top = '0';
  overlayCanvas.style.left = '0';
  overlayCanvas.style.width = '100%';
  overlayCanvas.style.height = '100%';
  pageWrapper.appendChild(overlayCanvas);

  const canvas = new fabric.Canvas(overlayCanvas, {
    selection: false,
    backgroundColor: 'rgba(0,0,0,0)'
  });

  const pxPerPt = (rendered.widthPx / page.widthPt);

  page.objects.forEach((object) => {
    const controller = createController(canvas, page, object, pxPerPt);
    updateMetadata(controller, matrixFromTarget(controller));
  });

  canvas.on('object:modified', (event) => {
    const target = event.target;
    if (!target) return;
    const metadata = (target as any).metadata as FabricMetadata | undefined;
    if (!metadata) return;

    const oldMatrix = metadata.baseMatrix;
    const newMatrix = matrixFromTarget(target);
    const delta = fabricDeltaToPdfDelta(oldMatrix, newMatrix, page.heightPt);

    const op: TransformPatch = {
      op: 'transform',
      target: { page: page.index, id: metadata.id },
      deltaMatrixPt: delta,
      kind: metadata.kind
    };

    callbacks.onTransform(op);
    updateMetadata(target, newMatrix);
  });

  return {
    dispose() {
      canvas.dispose();
      pageWrapper.removeChild(overlayCanvas);
    }
  };
}
