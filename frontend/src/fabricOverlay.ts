import type { fabric as FabricNamespace } from 'fabric';
import * as FabricNS from 'fabric';

const fabric = FabricNS as unknown as FabricNamespace;
import type { DocumentIR, ImageObject, Matrix, PageIR, PageObject, TextObject, TransformPatch } from './types';
import { fabricDeltaToPdfDelta, ptToPxMatrix, S, multiply } from './coords';

export type TransformDispatcher = (patch: TransformPatch) => Promise<boolean>;

type OverlayEntry = {
  canvas: fabric.Canvas;
  element: HTMLCanvasElement;
  page: PageIR;
};

type ControllerMeta = {
  id: string;
  kind: 'text' | 'image';
  page: number;
  baseMatrix: Matrix;
};

export class FabricOverlayManager {
  private overlays = new Map<number, OverlayEntry>();
  private dispatcher: TransformDispatcher | null = null;

  reset() {
    for (const entry of this.overlays.values()) {
      entry.canvas.dispose();
      entry.element.remove();
    }
    this.overlays.clear();
  }

  populate(
    ir: DocumentIR,
    wrappers: HTMLElement[],
    pageSizes: Array<{ width: number; height: number }>,
    dispatcher: TransformDispatcher,
  ) {
    this.reset();
    this.dispatcher = dispatcher;

    ir.pages.forEach((page) => {
      const wrapper = wrappers[page.index];
      const size = pageSizes[page.index];
      if (!wrapper || !size) return;
      const entry = this.mountOverlay(page, wrapper, size);
      page.objects.forEach((obj) => this.addController(entry, obj));
      entry.canvas.renderAll();
    });
  }

  private mountOverlay(page: PageIR, wrapper: HTMLElement, size: { width: number; height: number }): OverlayEntry {
    const canvasEl = document.createElement('canvas');
    canvasEl.width = size.width;
    canvasEl.height = size.height;
    canvasEl.style.width = `${size.width}px`;
    canvasEl.style.height = `${size.height}px`;
    canvasEl.className = 'fabric-page-overlay';
    wrapper.appendChild(canvasEl);

    const canvas = new fabric.Canvas(canvasEl, {
      selection: true,
      preserveObjectStacking: true,
    });

    canvas.on('object:modified', (event) => {
      if (!event.target || !this.dispatcher) {
        return;
      }
      void this.handleTransform(canvas, page, event.target as fabric.Object);
    });

    const entry: OverlayEntry = { canvas, element: canvasEl, page };
    this.overlays.set(page.index, entry);
    return entry;
  }

  private async handleTransform(canvas: fabric.Canvas, page: PageIR, target: fabric.Object) {
    const data = target.get('data') as ControllerMeta | undefined;
    if (!data || !this.dispatcher) {
      return;
    }
    const fold = target.calcTransformMatrix() as Matrix;
    const previous = data.baseMatrix.slice() as Matrix;
    const delta = fabricDeltaToPdfDelta(previous, fold, page.heightPt);
    const patch: TransformPatch = {
      op: 'transform',
      target: { page: page.index, id: data.id },
      deltaMatrixPt: delta,
      kind: data.kind,
    };

    const ok = await this.dispatcher(patch);
    if (ok) {
      data.baseMatrix = fold.slice() as Matrix;
      target.set('data', data);
      canvas.requestRenderAll();
    } else {
      target.set('transformMatrix', previous as unknown as number[]);
      target.setCoords();
      canvas.requestRenderAll();
    }
  }

  private addController(entry: OverlayEntry, obj: PageObject) {
    if (obj.kind === 'text') {
      this.addTextController(entry, obj);
    } else if (obj.kind === 'image') {
      this.addImageController(entry, obj);
    }
  }

  private addTextController(entry: OverlayEntry, obj: TextObject) {
    const { canvas, page } = entry;
    const [left, top, width, height] = this.toPixelRect(page, obj.bbox);
    const rect = new fabric.Rect({
      left,
      top,
      width,
      height,
      fill: 'rgba(0,0,0,0)',
      stroke: 'rgba(33, 150, 243, 0.6)',
      strokeDashArray: [8, 4],
      strokeWidth: 1,
      originX: 'left',
      originY: 'top',
      transparentCorners: false,
    });
    rect.set('transformMatrix', this.toFabricMatrix(page, obj.Tm));
    rect.set('data', {
      id: obj.id,
      kind: 'text',
      page: page.index,
      baseMatrix: rect.calcTransformMatrix() as Matrix,
    } satisfies ControllerMeta);
    rect.setCoords();
    canvas.add(rect);
  }

  private addImageController(entry: OverlayEntry, obj: ImageObject) {
    const { canvas, page } = entry;
    const [left, top, width, height] = this.toPixelRect(page, obj.bbox);
    const rect = new fabric.Rect({
      left,
      top,
      width,
      height,
      fill: 'rgba(0,0,0,0)',
      stroke: 'rgba(76, 175, 80, 0.6)',
      strokeDashArray: [8, 4],
      strokeWidth: 1,
      originX: 'left',
      originY: 'top',
      transparentCorners: false,
    });
    rect.set('transformMatrix', this.toFabricMatrix(page, obj.cm));
    rect.set('data', {
      id: obj.id,
      kind: 'image',
      page: page.index,
      baseMatrix: rect.calcTransformMatrix() as Matrix,
    } satisfies ControllerMeta);
    rect.setCoords();
    canvas.add(rect);
  }

  private toFabricMatrix(page: PageIR, matrixPt: Matrix) {
    const ptToPx = ptToPxMatrix(page.heightPt);
    return multiply(ptToPx, matrixPt as Matrix);
  }

  private toPixelRect(page: PageIR, bbox: [number, number, number, number]) {
    const [x0, y0, x1, y1] = bbox;
    const widthPt = x1 - x0;
    const heightPt = y1 - y0;
    const left = x0 / S;
    const top = (page.heightPt - y1) / S;
    const width = widthPt / S;
    const height = heightPt / S;
    return [left, top, width, height] as const;
  }
}
