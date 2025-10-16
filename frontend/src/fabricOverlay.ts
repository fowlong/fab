import type { Matrix, PageIR, PageObject } from './types';
import { fabricDeltaToPdfDelta, invert, multiply, pxToPtMatrix, S } from './coords';

type FabricNamespace = typeof import('fabric');

type OverlayMeta = {
  id: string;
  kind: 'text' | 'image';
  pageHeightPt: number;
  fold: Matrix;
};

type TransformHandler = (id: string, kind: 'text' | 'image', delta: Matrix) => Promise<void>;

type OverlayEntry = {
  canvas: any;
  element: HTMLCanvasElement;
  pageHeightPt: number;
};

export class FabricOverlayManager {
  private overlays = new Map<number, OverlayEntry>();
  private readonly fabricPromise: Promise<FabricNamespace['fabric']>;

  constructor(private readonly onTransform: TransformHandler) {
    this.fabricPromise = import('fabric').then((mod) => (mod as any).fabric ?? mod);
  }

  async reset() {
    for (const entry of this.overlays.values()) {
      entry.canvas.dispose();
      entry.element.remove();
    }
    this.overlays.clear();
  }

  async render(page: PageIR, wrapper: HTMLElement, size: { width: number; height: number }) {
    const fabric = await this.fabricPromise;
    const existing = this.overlays.get(page.index);
    if (existing) {
      existing.canvas.dispose();
      existing.element.remove();
      this.overlays.delete(page.index);
    }

    const canvasEl = document.createElement('canvas');
    canvasEl.width = size.width;
    canvasEl.height = size.height;
    canvasEl.style.width = `${size.width}px`;
    canvasEl.style.height = `${size.height}px`;
    canvasEl.className = 'fabric-page-overlay';
    wrapper.appendChild(canvasEl);

    const canvas = new fabric.Canvas(canvasEl, {
      preserveObjectStacking: true,
      selection: true,
    });

    this.overlays.set(page.index, {
      canvas,
      element: canvasEl,
      pageHeightPt: page.heightPt,
    });

    await Promise.all(page.objects.map((object) => this.addObject(fabric, canvas, page, object)));
    canvas.renderAll();
  }

  private async addObject(
    fabric: FabricNamespace['fabric'],
    canvas: any,
    page: PageIR,
    object: PageObject,
  ) {
    const bbox = object.bbox;
    const widthPt = bbox[2] - bbox[0];
    const heightPt = bbox[3] - bbox[1];
    const rect = new fabric.Rect({
      left: 0,
      top: 0,
      width: widthPt / S,
      height: heightPt / S,
      originX: 'left',
      originY: 'top',
      fill: 'rgba(0,0,0,0)',
      stroke: '#60a5fa',
      strokeWidth: 1,
      strokeDashArray: [6, 4],
      selectable: true,
      hasControls: true,
    });

    const baseMatrix = this.objectMatrix(object);
    const transform = this.toFabricMatrix(page.heightPt, baseMatrix);
    rect.transformMatrix = transform;

    const fold = rect.calcTransformMatrix() as Matrix;
    rect.set('data', {
      id: object.id,
      kind: object.kind,
      pageHeightPt: page.heightPt,
      fold,
    } satisfies OverlayMeta);

    rect.on('modified', () => {
      void this.handleModified(rect as any);
    });

    canvas.add(rect);
  }

  private objectMatrix(object: PageObject): Matrix {
    if (object.kind === 'text') {
      return object.Tm;
    }
    return object.cm;
  }

  private toFabricMatrix(pageHeightPt: number, matrixPt: Matrix): Matrix {
    const ptToPx = invert(pxToPtMatrix(pageHeightPt));
    return multiply(ptToPx, matrixPt);
  }

  private async handleModified(rect: any) {
    const data = rect.get('data') as OverlayMeta | undefined;
    if (!data) {
      return;
    }
    const fnew = rect.calcTransformMatrix() as Matrix;
    const delta = fabricDeltaToPdfDelta(data.fold, fnew, data.pageHeightPt);
    try {
      await this.onTransform(data.id, data.kind, delta);
      data.fold = fnew;
      rect.set('data', data);
    } catch (error) {
      console.error('Failed to apply transform', error);
      rect.set('transformMatrix', data.fold);
      rect.setCoords();
      rect.canvas?.renderAll();
    }
  }
}
