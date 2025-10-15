import { fabric } from 'fabric';
import type { DocumentIR, PageIR, PageObject, PatchOp } from './types';
import { fabricDeltaToPdfDelta, multiplyMatrix, type Matrix } from './coords';
import {
  createController,
  updateControllerGeometry,
  type FabricObjectWithMeta
} from './mapping';

export type OverlayCallbacks = {
  onTransform: (ops: PatchOp[]) => Promise<void>;
  onEditText: (obj: PageObject) => Promise<void>;
};

export class FabricOverlay {
  private canvases = new Map<number, fabric.Canvas>();
  private activeDocId: string | null = null;
  private callbacks: OverlayCallbacks;

  constructor(callbacks: OverlayCallbacks) {
    this.callbacks = callbacks;
  }

  attachToDocument(docId: string, ir: DocumentIR) {
    this.activeDocId = docId;
    this.dispose();
    ir.pages.forEach((page) => this.createCanvasForPage(page));
  }

  private createCanvasForPage(page: PageIR) {
    const canvasElement = document.createElement('canvas');
    canvasElement.id = `fabric-p${page.index}`;
    canvasElement.width = page.widthPt * (96 / 72);
    canvasElement.height = page.heightPt * (96 / 72);
    canvasElement.className = 'fabric-overlay-canvas';

    const host = document.querySelector(`#page-${page.index}`);
    if (!host) {
      return;
    }
    host.appendChild(canvasElement);

    const fabricCanvas = new fabric.Canvas(canvasElement, {
      selection: false,
      preserveObjectStacking: true
    });
    fabricCanvas.upperCanvasEl.style.pointerEvents = 'auto';
    fabricCanvas.lowerCanvasEl.style.pointerEvents = 'auto';

    page.objects.forEach((obj) => {
      const controller = createController(fabric, page, obj);
      fabricCanvas.add(controller);
      controller.on('modified', () => this.handleModified(controller));
      controller.on('mousedblclick', () => {
        void this.callbacks.onEditText(obj);
      });
    });

    this.canvases.set(page.index, fabricCanvas);
  }

  updatePage(page: PageIR) {
    const canvas = this.canvases.get(page.index);
    if (!canvas) return;

    canvas.getObjects().forEach((controller) => {
      const meta = (controller as FabricObjectWithMeta).__meta;
      if (!meta) {
        return;
      }
      const updated = page.objects.find((o) => o.id === meta.id);
      if (updated) {
        updateControllerGeometry(controller as FabricObjectWithMeta, page, updated);
      }
    });
  }

  private async handleModified(controller: FabricObjectWithMeta) {
    if (!this.activeDocId || !controller.__meta) {
      return;
    }
    const meta = controller.__meta;
    const canvas = this.canvases.get(meta.pageIndex);
    if (!canvas) return;

    const initialMatrix = meta.baseMatrix as Matrix;
    const current = controller.calcTransformMatrix() as Matrix;
    const delta = fabricDeltaToPdfDelta(initialMatrix, current, canvas.height * (72 / 96));

    const op: PatchOp = {
      op: 'transform',
      target: { page: meta.pageIndex, id: meta.id },
      deltaMatrixPt: delta,
      kind: meta.kind
    } as PatchOp;

    await this.callbacks.onTransform([op]);
    controller.__meta.baseMatrix = multiplyMatrix(delta, initialMatrix);
  }

  dispose() {
    this.canvases.forEach((canvas) => canvas.dispose());
    this.canvases.clear();
  }
}
