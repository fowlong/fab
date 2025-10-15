import { Canvas, Rect, util } from 'fabric';

import type { Matrix, PageIR, TextObject, ImageObject } from './types';
import { multiply, invert, pxToPtMatrix, fabricDeltaToPdfDelta, S } from './coords';

export type TransformHandler = (
  id: string,
  kind: 'text' | 'image',
  deltaMatrixPt: Matrix,
) => Promise<boolean>;

type OverlayMeta = {
  id: string;
  kind: 'text' | 'image';
  baseMatrix: Matrix;
  pageHeightPt: number;
};

export class FabricOverlayManager {
  private canvas: Canvas | null = null;
  private metas = new Map<Rect, OverlayMeta>();

  mount(container: HTMLElement, width: number, height: number) {
    this.dispose();
    const canvasEl = document.createElement('canvas');
    canvasEl.width = width;
    canvasEl.height = height;
    canvasEl.style.width = `${width}px`;
    canvasEl.style.height = `${height}px`;
    canvasEl.className = 'fabric-page-overlay';
    container.appendChild(canvasEl);
    this.canvas = new Canvas(canvasEl, {
      selection: false,
    });
  }

  async render(page: PageIR, handler: TransformHandler) {
    if (!this.canvas) {
      throw new Error('overlay not mounted');
    }
    this.canvas.clear();
    this.metas.clear();

    for (const object of page.objects) {
      if (object.kind === 'text') {
        this.addText(object, page, handler);
      } else if (object.kind === 'image') {
        this.addImage(object, page, handler);
      }
    }
    this.canvas.requestRenderAll();
  }

  dispose() {
    if (this.canvas) {
      this.canvas.dispose();
    }
    this.canvas = null;
    this.metas.clear();
  }

  private addText(object: TextObject, page: PageIR, handler: TransformHandler) {
    const baseMatrix = toFabricMatrix(object.Tm, page.heightPt);
    const rect = this.createRect(object.bbox, page.heightPt, baseMatrix);
    this.registerObject(rect, object.id, 'text', baseMatrix, page.heightPt, handler);
  }

  private addImage(object: ImageObject, page: PageIR, handler: TransformHandler) {
    const baseMatrix = toFabricMatrix(object.cm, page.heightPt);
    const rect = this.createRect(object.bbox, page.heightPt, baseMatrix);
    this.registerObject(rect, object.id, 'image', baseMatrix, page.heightPt, handler);
  }

  private createRect(bbox: [number, number, number, number], pageHeightPt: number, matrix: Matrix) {
    const [x0, y0, x1, y1] = bbox;
    const widthPx = (x1 - x0) / S;
    const heightPx = (y1 - y0) / S;
    const rect = new fabric.Rect({
      width: widthPx,
      height: heightPx,
      fill: 'rgba(59,130,246,0.08)',
      stroke: '#2563eb',
      strokeDashArray: [6, 4],
      strokeWidth: 1,
      selectable: true,
      hasBorders: true,
      hasControls: true,
      transparentCorners: false,
      originX: 'left',
      originY: 'top',
      centeredScaling: false,
      centeredRotation: false,
    });
    applyMatrixToObject(rect, matrix);
    return rect;
  }

  private registerObject(
    rect: Rect,
    id: string,
    kind: 'text' | 'image',
    baseMatrix: Matrix,
    pageHeightPt: number,
    handler: TransformHandler,
  ) {
    if (!this.canvas) {
      return;
    }
    this.canvas.add(rect);
    this.metas.set(rect, { id, kind, baseMatrix, pageHeightPt });
    rect.on('modified', () => {
      void this.handleModification(rect, handler);
    });
  }

  private async handleModification(target: Rect, handler: TransformHandler) {
    if (!this.canvas) {
      return;
    }
    const meta = this.metas.get(target);
    if (!meta) {
      return;
    }
    const nextMatrix = objectToMatrix(target);
    try {
      const delta = fabricDeltaToPdfDelta(meta.baseMatrix, nextMatrix, meta.pageHeightPt);
      const ok = await handler(meta.id, meta.kind, delta);
      if (ok) {
        meta.baseMatrix = nextMatrix;
        this.metas.set(target, meta);
      } else {
        applyMatrixToObject(target, meta.baseMatrix);
        this.canvas.requestRenderAll();
      }
    } catch (err) {
      console.error('Failed to compose transform', err);
      applyMatrixToObject(target, meta.baseMatrix);
      this.canvas.requestRenderAll();
    }
  }
}

function toFabricMatrix(matrixPt: Matrix, pageHeightPt: number): Matrix {
  const ptToPx = invert(pxToPtMatrix(pageHeightPt));
  return multiply(ptToPx, matrixPt);
}

function objectToMatrix(object: Rect): Matrix {
  const left = object.left ?? 0;
  const top = object.top ?? 0;
  const angleRad = util.degreesToRadians(object.angle ?? 0);
  const scaleX = object.scaleX ?? 1;
  const scaleY = object.scaleY ?? 1;
  const cos = Math.cos(angleRad);
  const sin = Math.sin(angleRad);
  return [
    cos * scaleX,
    sin * scaleX,
    -sin * scaleY,
    cos * scaleY,
    left,
    top,
  ];
}

function applyMatrixToObject(object: Rect, matrix: Matrix) {
  const [a, b, c, d, e, f] = matrix;
  const scaleX = Math.sqrt(a * a + b * b) || 1;
  const scaleY = Math.sqrt(c * c + d * d) || 1;
  const angle = Math.atan2(b, a) * (180 / Math.PI);
  object.set({
    left: e,
    top: f,
    angle,
    scaleX,
    scaleY,
    skewX: 0,
    skewY: 0,
    originX: 'left',
    originY: 'top',
  });
  object.setCoords();
}
