import { Canvas, Rect, FabricObject } from 'fabric';

import type { DocumentIR, ImageObject, PageIR, PageObject, TextObject } from './types';
import type { Matrix } from './types';
import { bboxPtToPx, multiply, ptToPxMatrix } from './coords';

type OverlayCanvas = {
  canvas: Canvas;
  pageHeightPt: number;
};

type ControllerMeta = {
  id: string;
  kind: 'text' | 'image';
  pageIndex: number;
  fold: Matrix;
  pageHeightPt: number;
};

type TransformRequest = {
  id: string;
  kind: 'text' | 'image';
  fold: Matrix;
  next: Matrix;
  pageHeightPt: number;
};

type TransformCallback = (request: TransformRequest) => Promise<boolean>;

export class FabricOverlayManager {
  private overlays = new Map<number, OverlayCanvas>();
  private readonly onTransform: TransformCallback;

  constructor(onTransform: TransformCallback) {
    this.onTransform = onTransform;
  }

  reset(): void {
    for (const entry of this.overlays.values()) {
      entry.canvas.dispose();
    }
    this.overlays.clear();
  }

  populate(ir: DocumentIR, wrappers: HTMLElement[], pageSizes: Array<{ width: number; height: number }>): void {
    this.reset();
    ir.pages.forEach((page) => {
      const wrapper = wrappers[page.index];
      const size = pageSizes[page.index];
      if (!wrapper || !size) {
        return;
      }
      const canvas = this.createCanvas(page, wrapper, size);
      page.objects.forEach((obj) => this.addController(canvas, page, obj));
      canvas.renderAll();
    });
  }

  private createCanvas(page: PageIR, wrapper: HTMLElement, size: { width: number; height: number }): Canvas {
    const canvasEl = document.createElement('canvas');
    canvasEl.width = size.width;
    canvasEl.height = size.height;
    canvasEl.style.width = `${size.width}px`;
    canvasEl.style.height = `${size.height}px`;
    canvasEl.className = 'fabric-page-overlay';
    wrapper.appendChild(canvasEl);

    const canvas = new Canvas(canvasEl, {
      selection: false,
      preserveObjectStacking: true,
    });

    canvas.on('object:modified', (event) => {
      const target = event.target as FabricObject | undefined;
      if (!target) {
        return;
      }
      const meta = target.data as ControllerMeta | undefined;
      if (!meta) {
        return;
      }
      const current = extractMatrix(target);
      const fold = meta.fold;
      void this.onTransform({
        id: meta.id,
        kind: meta.kind,
        fold,
        next: current,
        pageHeightPt: meta.pageHeightPt,
      })
        .then((ok) => {
          if (ok) {
            meta.fold = current;
            target.set('data', meta);
          } else {
            applyMatrix(target, fold);
            target.canvas?.renderAll();
          }
        })
        .catch((error) => {
          console.error(error);
          applyMatrix(target, fold);
          target.canvas?.renderAll();
        });
    });

    this.overlays.set(page.index, { canvas, pageHeightPt: page.heightPt });
    return canvas;
  }

  private addController(canvas: Canvas, page: PageIR, object: PageObject): void {
    if (object.kind === 'text') {
      const rect = this.createController(canvas, page, object, object.Tm, 'text');
      canvas.add(rect);
    } else if (object.kind === 'image') {
      const rect = this.createController(canvas, page, object, object.cm, 'image');
      canvas.add(rect);
    }
  }

  private createController(
    canvas: Canvas,
    page: PageIR,
    object: TextObject | ImageObject,
    matrixPt: Matrix,
    kind: 'text' | 'image',
  ): Rect {
    const [, , width, height] = bboxPtToPx(page.heightPt, object.bbox);
    const rect = new Rect({
      left: 0,
      top: 0,
      width,
      height,
      fill: 'rgba(37, 99, 235, 0.08)',
      stroke: '#2563eb',
      strokeWidth: 1,
      strokeDashArray: [6, 4],
      originX: 'left',
      originY: 'top',
      hasBorders: false,
      cornerColor: '#1d4ed8',
      transparentCorners: false,
    });

    const transform = toFabricMatrix(page.heightPt, matrixPt);
    applyMatrix(rect, transform);

    const meta: ControllerMeta = {
      id: object.id,
      kind,
      pageIndex: page.index,
      fold: transform,
      pageHeightPt: page.heightPt,
    };
    rect.set('data', meta);
    rect.setControlsVisibility({ mtr: true });
    rect.on('mousedown', () => canvas.setActiveObject(rect));
    return rect;
  }
}

function toFabricMatrix(pageHeightPt: number, matrixPt: Matrix): Matrix {
  const ptToPx = ptToPxMatrix(pageHeightPt);
  return multiply(ptToPx, matrixPt);
}

function extractMatrix(object: FabricObject): Matrix {
  const values = object.calcTransformMatrix();
  return [values[0], values[1], values[4], values[5], values[12], values[13]];
}

function applyMatrix(object: FabricObject, matrix: Matrix): void {
  object.set({
    transformMatrix: matrix as unknown as number[],
    left: 0,
    top: 0,
  });
  object.dirty = true;
}
