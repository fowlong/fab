import { fabric } from 'fabric';
import type { Matrix } from './coords';
import { fabricDeltaToPdfDelta } from './coords';
import type { DocumentIR, PageIR, PageObject, PatchOp } from './types';
import { bboxToFabricRect, objectBoundingMatrix } from './mapping';

export type FabricOverlay = {
  canvases: fabric.Canvas[];
  mount(pageRoot: HTMLElement, page: PageIR): fabric.Canvas;
  update(ir: DocumentIR): void;
  destroy(): void;
  setPatchHandler(handler: (ops: PatchOp[]) => Promise<void>): void;
};

type OverlayEntry = {
  fabricObject: fabric.Rect;
  meta: {
    id: string;
    pageIndex: number;
    baseMatrix: Matrix;
  };
};

export function createFabricOverlay(): FabricOverlay {
  const canvases: fabric.Canvas[] = [];
  const overlayEntries = new Map<string, OverlayEntry>();
  let patchHandler: (ops: PatchOp[]) => Promise<void> = async () => {};

  function mount(pageRoot: HTMLElement, page: PageIR) {
    const canvasEl = document.createElement('canvas');
    canvasEl.id = `fabric-page-${page.index}`;
    canvasEl.width = pageRoot.clientWidth;
    canvasEl.height = pageRoot.clientHeight;
    canvasEl.className = 'fabric-overlay';
    pageRoot.appendChild(canvasEl);
    const canvas = new fabric.Canvas(canvasEl, {
      selection: false,
      preserveObjectStacking: true,
    });
    canvases.push(canvas);
    return canvas;
  }

  function populatePage(canvas: fabric.Canvas, page: PageIR) {
    canvas.clear();
    const entries: OverlayEntry[] = [];
    for (const object of page.objects) {
      const rect = bboxToFabricRect(object.bbox, page.heightPt);
      const controller = new fabric.Rect({
        left: rect.left,
        top: rect.top,
        width: rect.width,
        height: rect.height,
        fill: 'rgba(0,0,0,0)',
        stroke: '#1d4ed8',
        strokeDashArray: [6, 4],
        hasBorders: false,
        hasControls: true,
        objectCaching: false,
      });
      const baseMatrix = objectBoundingMatrix(object, page.heightPt);
      controller.matrixCache = baseMatrix;
      (controller as any).irMeta = { id: object.id, pageIndex: page.index, baseMatrix };
      controller.on('modified', () => {
        const meta = (controller as any).irMeta as OverlayEntry['meta'];
        const ops: PatchOp[] = [];
        const newTransform = extractMatrix(controller);
        const delta = fabricDeltaToPdfDelta(meta.baseMatrix, newTransform, page.heightPt);
        const patchKind: PageObject['kind'] = object.kind;
        ops.push({
          op: 'transform',
          target: { page: page.index, id: object.id },
          deltaMatrixPt: delta,
          kind: patchKind,
        });
        patchHandler(ops).catch((err) => console.error('Patch failed', err));
      });
      canvas.add(controller);
      entries.push({ fabricObject: controller, meta: { id: object.id, pageIndex: page.index, baseMatrix } });
      overlayEntries.set(object.id, entries[entries.length - 1]);
    }
  }

  function update(ir: DocumentIR) {
    ir.pages.forEach((page, index) => {
      const canvas = canvases[index];
      if (canvas) {
        populatePage(canvas, page);
      }
    });
  }

  function destroy() {
    canvases.forEach((canvas) => canvas.dispose());
    canvases.length = 0;
    overlayEntries.clear();
  }

  function extractMatrix(obj: fabric.Object): Matrix {
    const { a, b, c, d, e, f } = obj.calcTransformMatrix();
    return [a, b, c, d, e, f];
  }

  return {
    canvases,
    mount,
    update,
    destroy,
    setPatchHandler(handler) {
      patchHandler = handler;
    },
  };
}
