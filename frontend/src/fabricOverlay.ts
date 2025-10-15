import { fabric } from 'fabric';
import { fabricDeltaToPdfDelta, pdfMatrixToFabric, S } from './coords';
import type { Matrix, PageIR, PageObject } from './types';

type TransformCallback = (id: string, kind: 'text' | 'image', delta: Matrix) => Promise<void>;
type ErrorCallback = (error: unknown) => void;

type ControllerMeta = {
  id: string;
  kind: 'text' | 'image';
  pageIndex: number;
  pageHeightPt: number;
  fold: Matrix;
};

export class FabricOverlay {
  private canvas: fabric.Canvas | null = null;
  private container: HTMLElement;
  private readonly onTransform: TransformCallback;
  private readonly onError?: ErrorCallback;

  constructor(container: HTMLElement, onTransform: TransformCallback, onError?: ErrorCallback) {
    this.container = container;
    this.onTransform = onTransform;
    this.onError = onError;
  }

  clear(): void {
    if (this.canvas) {
      this.canvas.dispose();
      this.canvas = null;
    }
    this.container.innerHTML = '';
  }

  render(page: PageIR): void {
    this.clear();
    const canvasEl = document.createElement('canvas');
    const widthPx = page.widthPt / S;
    const heightPx = page.heightPt / S;
    canvasEl.width = widthPx;
    canvasEl.height = heightPx;
    canvasEl.style.width = `${widthPx}px`;
    canvasEl.style.height = `${heightPx}px`;
    canvasEl.className = 'overlay-canvas';
    this.container.appendChild(canvasEl);

    const canvas = new fabric.Canvas(canvasEl, {
      selection: false,
      preserveObjectStacking: true,
    });
    this.canvas = canvas;

    page.objects.forEach((object) => {
      const controller = this.createController(page, object);
      canvas.add(controller);
    });

    canvas.on('object:modified', (event) => {
      const target = event.target as fabric.Object & { data?: ControllerMeta };
      if (!target || !target.data) {
        return;
      }
      this.commitTransform(target).catch((error: unknown) => {
        if (this.onError) {
          this.onError(error);
        } else {
          console.error('Patch failed', error);
        }
      });
    });
  }

  private createController(page: PageIR, object: PageObject): fabric.Rect {
    const widthPx = object.bboxPt[2] / S;
    const heightPx = object.bboxPt[3] / S;
    const transformPt = object.kind === 'text' ? object.Tm : object.cm;
    const matrixPx = pdfMatrixToFabric(transformPt, page.heightPt);

    const rect = new fabric.Rect({
      width: widthPx,
      height: heightPx,
      fill: 'rgba(59, 130, 246, 0.12)',
      stroke: '#1d4ed8',
      strokeDashArray: [6, 4],
      strokeWidth: 1,
      selectable: true,
      hasBorders: false,
      originX: 'left',
      originY: 'top',
      transparentCorners: false,
      cornerColor: '#1d4ed8',
    });

    rect.set('transformMatrix', matrixPx);
    rect.set('data', {
      id: object.id,
      kind: object.kind,
      pageIndex: page.index,
      pageHeightPt: page.heightPt,
      fold: matrixPx,
    } satisfies ControllerMeta);
    rect.setCoords();
    return rect;
  }

  private async commitTransform(target: fabric.Object & { data?: ControllerMeta }): Promise<void> {
    if (!this.canvas || !target.data) {
      return;
    }
    const meta = target.data;
    const fold = meta.fold;
    const fnew = [...(target.calcTransformMatrix() as number[])] as Matrix;
    try {
      const delta = fabricDeltaToPdfDelta(fold, fnew, meta.pageHeightPt);
      await this.onTransform(meta.id, meta.kind, delta);
      meta.fold = fnew;
      target.set('data', meta);
      target.setCoords();
      this.canvas.requestRenderAll();
    } catch (error) {
      target.set('transformMatrix', fold);
      target.setCoords();
      this.canvas.requestRenderAll();
      throw error;
    }
  }
}
