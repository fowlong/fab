import { Canvas, Rect } from 'fabric';

import type { Matrix, DocumentIR, PageObject } from './types';
import { fabricDeltaToPdfDelta, ptMatrixToFabric, S } from './coords';
import { patch } from './api';

export type OverlayCallbacks = {
  onUpdatedPdf?: (dataUrl?: string) => void;
  onError?: (message: string) => void;
  onInfo?: (message: string) => void;
};

const DEFAULT_STROKE = 'rgba(41,128,185,0.7)';
const DEFAULT_FILL = 'rgba(41,128,185,0.08)';

export class FabricOverlay {
  private canvas: Canvas | null = null;
  private docId: string | null = null;
  private pageHeightPt = 0;
  private callbacks: OverlayCallbacks;

  constructor(callbacks: OverlayCallbacks = {}) {
    this.callbacks = callbacks;
  }

  dispose() {
    if (this.canvas) {
      this.canvas.dispose();
      this.canvas = null;
    }
    this.docId = null;
  }

  mount(
    element: HTMLCanvasElement,
    sizePx: { width: number; height: number },
    docId: string,
    pageHeightPt: number,
    ir: DocumentIR,
  ) {
    this.dispose();
    this.docId = docId;
    this.pageHeightPt = pageHeightPt;

    element.width = sizePx.width;
    element.height = sizePx.height;
    element.style.width = `${sizePx.width}px`;
    element.style.height = `${sizePx.height}px`;

    this.canvas = new Canvas(element, { selection: true });
    this.canvas.on('object:modified', (event: any) => {
      if (!event.target) return;
      this.handleModified(event.target).catch((err) => {
        this.callbacks.onError?.(String(err));
      });
    });

    const page = ir.pages.find((p) => p.index === 0);
    if (!page) {
      return;
    }
    page.objects.forEach((object) => this.addController(object));
    this.canvas.renderAll();
  }

  private addController(obj: PageObject) {
    if (!this.canvas || !this.docId) {
      return;
    }
    const { widthPt, heightPt } = getBoxSize(obj);
    const widthPx = widthPt / S;
    const heightPx = heightPt / S;
    const rect = new Rect({
      width: widthPx,
      height: heightPx,
      fill: DEFAULT_FILL,
      stroke: DEFAULT_STROKE,
      strokeWidth: 1,
      transparentCorners: false,
      cornerColor: '#2980b9',
      lockScalingFlip: true,
      originX: 'left',
      originY: 'top',
      selectable: true,
    });

    const pdfMatrix = getMatrix(obj);
    const fabricMatrix = ptMatrixToFabric(pdfMatrix, this.pageHeightPt);
    rect.set('transformMatrix', fabricMatrix);
    rect.set('data', {
      id: obj.id,
      kind: obj.kind,
      page: 0,
      F0: fabricMatrix,
    });
    rect.set('hoverCursor', 'move');
    rect.set('strokeUniform', true);

    this.canvas.add(rect);
  }

  private async handleModified(target: any) {
    if (!this.canvas || !this.docId) {
      return;
    }
    const meta = target.data as { id: string; kind: 'text' | 'image'; page: number; F0: Matrix } | undefined;
    if (!meta) {
      return;
    }
    const Fnew: Matrix = target.calcTransformMatrix();
    try {
      const delta = fabricDeltaToPdfDelta(meta.F0, Fnew, this.pageHeightPt);
      const response = await patch(this.docId, [
        {
          op: 'transform',
          target: { page: meta.page, id: meta.id },
          deltaMatrixPt: delta,
          kind: meta.kind,
        },
      ]);
      if (!response.ok) {
        throw new Error('Patch rejected by backend');
      }
      meta.F0 = Fnew;
      target.set('data', meta);
      target.setCoords();
      this.canvas.renderAll();
      if (response.updatedPdf) {
        this.callbacks.onUpdatedPdf?.(response.updatedPdf);
      } else {
        this.callbacks.onInfo?.('Updated PDF ready.');
      }
    } catch (error) {
      target.set('transformMatrix', meta.F0);
      target.setCoords();
      this.canvas.renderAll();
      throw error;
    }
  }
}

function getMatrix(obj: PageObject): Matrix {
  if (obj.kind === 'text') {
    return obj.Tm;
  }
  return obj.cm;
}

function getBoxSize(obj: PageObject): { widthPt: number; heightPt: number } {
  const [x0, y0, x1, y1] = obj.bbox;
  return { widthPt: Math.max(1, x1 - x0), heightPt: Math.max(1, y1 - y0) };
}
