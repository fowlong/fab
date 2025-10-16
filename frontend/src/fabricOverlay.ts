import { fabric } from 'fabric';
import type { Matrix, DocumentIR, ImageObject, TextObject } from './types';
import { fabricDeltaToPdfDelta, invert, multiply, pxToPtMatrix, S } from './coords';

export type TransformHandler = (
  payload: {
    id: string;
    kind: 'text' | 'image';
    delta: Matrix;
  },
) => Promise<void>;

type ControllerMeta = {
  id: string;
  kind: 'text' | 'image';
  base: Matrix;
};

export class FabricOverlay {
  private canvas: fabric.Canvas | null = null;
  private pageHeightPt = 0;
  private onTransform: TransformHandler | null = null;
  private isPatching = false;

  initialise(container: HTMLElement, widthPx: number, heightPx: number): void {
    container.innerHTML = '';
    const canvasEl = document.createElement('canvas');
    canvasEl.width = widthPx;
    canvasEl.height = heightPx;
    canvasEl.style.width = `${widthPx}px`;
    canvasEl.style.height = `${heightPx}px`;
    container.appendChild(canvasEl);
    this.canvas = new fabric.Canvas(canvasEl, {
      selection: false,
      preserveObjectStacking: true,
    });
  }

  async render(
    ir: DocumentIR,
    metrics: { widthPx: number; heightPx: number; heightPt: number },
    onTransform: TransformHandler,
  ): Promise<void> {
    if (!this.canvas) {
      throw new Error('Overlay canvas not initialised');
    }
    this.canvas.clear();
    this.canvas.off('object:modified');
    this.onTransform = onTransform;
    this.pageHeightPt = metrics.heightPt;

    const ptToPx = invert(pxToPtMatrix(metrics.heightPt));

    for (const object of ir.pages[0]?.objects ?? []) {
      if (object.kind === 'text') {
        this.addTextController(object, ptToPx);
      } else if (object.kind === 'image') {
        this.addImageController(object, ptToPx);
      }
    }

    this.canvas.on('object:modified', async (event) => {
      if (this.isPatching) {
        return;
      }
      const target = event.target as fabric.Object | undefined;
      if (!target) {
        return;
      }
      const meta = target.data as ControllerMeta | undefined;
      if (!meta || !this.onTransform) {
        return;
      }
      const currentMatrix = target.calcTransformMatrix();
      const delta = fabricDeltaToPdfDelta(meta.base, currentMatrix as Matrix, this.pageHeightPt);
      this.isPatching = true;
      try {
        await this.onTransform({ id: meta.id, kind: meta.kind, delta });
        meta.base = currentMatrix as Matrix;
      } finally {
        this.isPatching = false;
      }
    });
  }

  private addTextController(object: TextObject, ptToPx: Matrix): void {
    if (!this.canvas) {
      return;
    }
    const matrix = multiply(ptToPx, object.Tm);
    const widthPt = object.bbox ? object.bbox[2] - object.bbox[0] : object.font.size * 6;
    const heightPt = object.bbox ? object.bbox[3] - object.bbox[1] : object.font.size * 1.2;
    const rect = this.createController(widthPt, heightPt, matrix, object.id, 'text');
    this.canvas.add(rect);
  }

  private addImageController(object: ImageObject, ptToPx: Matrix): void {
    if (!this.canvas) {
      return;
    }
    const matrix = multiply(ptToPx, object.cm);
    const widthPt = object.bbox ? object.bbox[2] - object.bbox[0] : 100;
    const heightPt = object.bbox ? object.bbox[3] - object.bbox[1] : 100;
    const rect = this.createController(widthPt, heightPt, matrix, object.id, 'image');
    this.canvas.add(rect);
  }

  private createController(
    widthPt: number,
    heightPt: number,
    transformMatrix: Matrix,
    id: string,
    kind: 'text' | 'image',
  ): fabric.Rect {
    const widthPx = widthPt / S;
    const heightPx = heightPt / S;
    const rect = new fabric.Rect({
      width: widthPx,
      height: heightPx,
      fill: 'rgba(33, 150, 243, 0.08)',
      stroke: '#2196f3',
      strokeWidth: 1,
      strokeDashArray: [6, 4],
      selectable: true,
      hasBorders: false,
      hasControls: true,
      lockScalingFlip: true,
      originX: 'left',
      originY: 'top',
    });
    rect.set('transformMatrix', transformMatrix);
    rect.set('data', { id, kind, base: rect.calcTransformMatrix() } satisfies ControllerMeta);
    return rect;
  }
}
