import * as FabricNS from 'fabric';
import type { fabric } from 'fabric';
import type { DocumentIR, PageIR, PageObject, PatchOperation, PatchResponse } from './types';
import { fabricDeltaToPdfDelta, invert, multiply, pxToPtMatrix, type Matrix, S } from './coords';

const fabric: typeof FabricNS.fabric =
  (FabricNS as any).fabric ??
  (FabricNS as any).default ??
  (FabricNS as any);

type PatchHandler = (ops: PatchOperation[]) => Promise<PatchResponse>;

type ControllerMeta = {
  id: string;
  kind: 'text' | 'image';
  page: number;
  baseMatrix: Matrix;
  pageHeightPt: number;
};

type OverlayEntry = {
  canvas: fabric.Canvas;
  element: HTMLCanvasElement;
};

export class FabricOverlayManager {
  private overlays = new Map<number, OverlayEntry>();
  private patcher: PatchHandler | null = null;

  reset() {
    for (const entry of this.overlays.values()) {
      entry.canvas.dispose();
      entry.element.remove();
    }
    this.overlays.clear();
  }

  async populate(
    ir: DocumentIR,
    wrappers: HTMLElement[],
    pageSizes: Array<{ width: number; height: number }>,
    patcher: PatchHandler,
  ) {
    this.reset();
    this.patcher = patcher;

    ir.pages.forEach((page) => {
      const wrapper = wrappers[page.index];
      const size = pageSizes[page.index];
      if (!wrapper || !size) {
        return;
      }
      const canvas = this.mountOverlay(page, wrapper, size);
      page.objects.forEach((object) => this.addController(canvas, page, object));
      canvas.renderAll();
    });
  }

  private mountOverlay(
    page: PageIR,
    wrapper: HTMLElement,
    size: { width: number; height: number },
  ) {
    const existing = this.overlays.get(page.index);
    if (existing) {
      existing.canvas.dispose();
      existing.element.remove();
    }

    const canvasEl = document.createElement('canvas');
    canvasEl.width = size.width;
    canvasEl.height = size.height;
    canvasEl.style.width = `${size.width}px`;
    canvasEl.style.height = `${size.height}px`;
    canvasEl.className = 'fabric-page-overlay';
    wrapper.appendChild(canvasEl);

    const canvas = new fabric.Canvas(canvasEl, {
      selection: true,
    });

    this.overlays.set(page.index, { canvas, element: canvasEl });
    return canvas;
  }

  private addController(canvas: fabric.Canvas, page: PageIR, object: PageObject) {
    const rect = new fabric.Rect({
      width: Math.max(this.bboxWidthPx(object), 4),
      height: Math.max(this.bboxHeightPx(object), 4),
      fill: 'rgba(59, 130, 246, 0.12)',
      stroke: '#2563eb',
      strokeDashArray: [6, 4],
      strokeWidth: 1,
      originX: 'left',
      originY: 'top',
      selectable: true,
      evented: true,
    });

    const pdfMatrix = this.objectMatrix(object);
    const fabricMatrix = this.pdfToFabricMatrix(pdfMatrix, page.heightPt);
    applyMatrix(rect, fabricMatrix);

    const meta: ControllerMeta = {
      id: object.id,
      kind: object.kind,
      page: page.index,
      baseMatrix: fabricMatrix,
      pageHeightPt: page.heightPt,
    };
    rect.set('data', meta);

    rect.on('modified', () => {
      void this.handleModified(rect);
    });

    canvas.add(rect);
    rect.setCoords();
  }

  private bboxWidthPx(object: PageObject) {
    const [x0, , x1] = object.bbox;
    return Math.abs(x1 - x0) / S;
  }

  private bboxHeightPx(object: PageObject) {
    const [, y0, , y1] = object.bbox;
    return Math.abs(y1 - y0) / S;
  }

  private objectMatrix(object: PageObject): Matrix {
    if (object.kind === 'text') {
      return object.Tm;
    }
    return object.cm;
  }

  private pdfToFabricMatrix(matrix: Matrix, pageHeightPt: number): Matrix {
    const ptToPx = invert(pxToPtMatrix(pageHeightPt));
    return multiply(ptToPx, matrix);
  }

  private async handleModified(rect: fabric.Rect) {
    if (!this.patcher) {
      return;
    }
    const meta = rect.get('data') as ControllerMeta | undefined;
    if (!meta) {
      return;
    }
    const currentMatrix = fabricMatrixFromObject(rect);
    try {
      const delta = fabricDeltaToPdfDelta(meta.baseMatrix, currentMatrix, meta.pageHeightPt);
      const op: PatchOperation = {
        op: 'transform',
        target: { page: meta.page, id: meta.id },
        deltaMatrixPt: delta,
        kind: meta.kind,
      };
      const response = await this.patcher([op]);
      if (!response.ok) {
        throw new Error('Patch rejected by backend');
      }
      meta.baseMatrix = currentMatrix;
      rect.set('data', meta);
    } catch (err) {
      applyMatrix(rect, meta.baseMatrix);
      rect.setCoords();
      console.error('Failed to apply patch', err);
    }
  }
}

function applyMatrix(object: fabric.Rect, matrix: Matrix) {
  const options = matrixToOptions(matrix);
  object.set({
    left: options.translateX,
    top: options.translateY,
    scaleX: options.scaleX,
    scaleY: options.scaleY,
    angle: options.angle,
    skewX: options.skewX,
    skewY: options.skewY,
  });
}

function matrixToOptions(matrix: Matrix) {
  const transform = [
    [matrix[0], matrix[2], matrix[4]],
    [matrix[1], matrix[3], matrix[5]],
    [0, 0, 1],
  ];
  const util = (fabric.util as unknown as { qrDecompose: (mat: number[][]) => any }).qrDecompose;
  const decomposed = util(transform);
  return decomposed;
}

function fabricMatrixFromObject(object: fabric.Object): Matrix {
  const transform = object.calcTransformMatrix();
  return [
    transform[0][0],
    transform[1][0],
    transform[0][1],
    transform[1][1],
    transform[0][2],
    transform[1][2],
  ];
}
