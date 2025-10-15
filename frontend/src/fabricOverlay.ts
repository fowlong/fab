import * as FabricNS from 'fabric';
const fabric: typeof import('fabric')['fabric'] =
  (FabricNS as any).fabric ?? (FabricNS as any).default ?? (FabricNS as any);

import type { Matrix } from './coords';
import { fabricDeltaToPdfDelta, multiply, pxToPtMatrix, ptBboxToPx } from './coords';
import type { PageIR, PageObject, PatchOperation } from './types';

export type OverlayCallbacks = {
  onTransform: (op: PatchOperation) => Promise<boolean>;
};

type OverlayMeta = {
  id: string;
  kind: 'text' | 'image';
  page: number;
  F0: Matrix;
};

export class FabricOverlay {
  private canvas: fabric.Canvas;
  private pageHeightPt: number;
  private callbacks: OverlayCallbacks;

  constructor(container: HTMLElement, size: { width: number; height: number }, pageHeightPt: number, callbacks: OverlayCallbacks) {
    this.pageHeightPt = pageHeightPt;
    this.callbacks = callbacks;

    const canvasEl = document.createElement('canvas');
    canvasEl.width = size.width;
    canvasEl.height = size.height;
    canvasEl.style.width = `${size.width}px`;
    canvasEl.style.height = `${size.height}px`;
    canvasEl.className = 'fabric-page-overlay';
    container.innerHTML = '';
    container.appendChild(canvasEl);

    this.canvas = new fabric.Canvas(canvasEl, {
      selection: false,
      preserveObjectStacking: true,
    });

    this.canvas.on('object:modified', (event) => {
      const target = event.target as fabric.Object & { data?: OverlayMeta };
      if (!target || !target.data) {
        return;
      }
      void this.handleTransform(target);
    });
  }

  dispose() {
    this.canvas.dispose();
  }

  render(page: PageIR) {
    this.canvas.clear();
    for (const object of page.objects) {
      this.addObject(page, object);
    }
    this.canvas.renderAll();
  }

  private addObject(page: PageIR, object: PageObject) {
    const pageHeightPt = page.heightPt;
    const bbox = object.kind === 'text' ? object.bbox : object.bbox;
    const [, , widthPx, heightPx] = ptBboxToPx(pageHeightPt, bbox);

    const rect = new fabric.Rect({
      left: 0,
      top: 0,
      width: widthPx,
      height: heightPx,
      fill: 'rgba(59, 130, 246, 0.08)',
      stroke: '#60a5fa',
      strokeWidth: 1,
      strokeDashArray: [6, 4],
      selectable: true,
      hasControls: true,
      objectCaching: false,
      transparentCorners: false,
      cornerColor: '#1d4ed8',
      originX: 'left',
      originY: 'top',
    });

    const matrixPt = object.kind === 'text' ? object.Tm : object.cm;
    const fabricMatrix = multiply(ptToPxMatrix(page.heightPt), matrixPt);
    rect.set('transformMatrix', fabricMatrix as unknown as number[]);

    rect.set('data', {
      id: object.id,
      kind: object.kind,
      page: page.index,
      F0: rect.calcTransformMatrix() as Matrix,
    } satisfies OverlayMeta);
    this.canvas.add(rect);
  }

  private async handleTransform(target: fabric.Object & { data?: OverlayMeta }) {
    if (!target.data) {
      return;
    }
    const meta = target.data;
    const Fnew = target.calcTransformMatrix() as Matrix;
    const delta = fabricDeltaToPdfDelta(meta.F0, Fnew, this.pageHeightPt);
    const op: PatchOperation = {
      op: 'transform',
      target: { page: meta.page, id: meta.id },
      deltaMatrixPt: delta,
      kind: meta.kind,
    };

    try {
      const ok = await this.callbacks.onTransform(op);
      if (ok) {
        meta.F0 = Fnew;
      } else {
        target.set('transformMatrix', meta.F0 as unknown as number[]);
        this.canvas.renderAll();
      }
    } catch (err) {
      console.error('Transform failed', err);
      target.set('transformMatrix', meta.F0 as unknown as number[]);
      this.canvas.renderAll();
    }
  }
}
