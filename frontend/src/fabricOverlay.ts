import type { ImageObject, PageIR, PageObject, TextObject } from './types';
import { fabricDeltaToPdfDelta, invert, multiply, pxToPtMatrix, ptToPx } from './coords';
import type { Matrix } from './coords';

import * as FabricNS from 'fabric';

const fabric: typeof FabricNS.fabric =
  (FabricNS as any).fabric ?? (FabricNS as any).default ?? (FabricNS as any);

type OverlayCallback = (patch: {
  id: string;
  kind: 'text' | 'image';
  delta: Matrix;
}) => Promise<void>;

type ControllerMeta = {
  id: string;
  kind: 'text' | 'image';
  fold: Matrix;
};

export class FabricOverlayManager {
  private canvas: fabric.Canvas | null = null;
  private pageHeightPt = 0;
  private callback: OverlayCallback | null = null;
  private busy = false;

  mount(container: HTMLElement, width: number, height: number) {
    if (this.canvas) {
      this.canvas.dispose();
    }
    const canvasElement = document.createElement('canvas');
    canvasElement.width = width;
    canvasElement.height = height;
    canvasElement.className = 'fabric-overlay';
    canvasElement.style.width = `${width}px`;
    canvasElement.style.height = `${height}px`;
    container.innerHTML = '';
    container.appendChild(canvasElement);
    this.canvas = new fabric.Canvas(canvasElement, {
      selection: false,
      preserveObjectStacking: true,
    });
  }

  render(page: PageIR, callback: OverlayCallback) {
    if (!this.canvas) {
      throw new Error('Overlay canvas has not been mounted');
    }
    this.canvas.clear();
    this.callback = callback;
    this.pageHeightPt = page.heightPt;
    page.objects.forEach((object) => this.addController(object, page));
    this.canvas.renderAll();
  }

  private addController(object: PageObject, page: PageIR) {
    if (!this.canvas) return;
    const matrixPt = object.kind === 'text' ? (object as TextObject).Tm : (object as ImageObject).cm;
    const widthPt = object.bbox[2] - object.bbox[0];
    const heightPt = object.bbox[3] - object.bbox[1];
    const widthPx = ptToPx(widthPt);
    const heightPx = ptToPx(heightPt);
    const rect = new fabric.Rect({
      left: 0,
      top: 0,
      width: widthPx,
      height: heightPx,
      fill: 'rgba(0,0,0,0)',
      stroke: 'rgba(59,130,246,0.7)',
      strokeDashArray: [6, 4],
      strokeWidth: 1,
      originX: 'left',
      originY: 'top',
      transparentCorners: false,
      hasBorders: false,
    });
    const ptToPxMatrix = invert(pxToPtMatrix(page.heightPt));
    const transformPx = multiply(ptToPxMatrix, matrixPt);
    rect.set('transformMatrix', transformPx);
    rect.set('data', {
      id: object.id,
      kind: object.kind,
      fold: rect.calcTransformMatrix(),
    } satisfies ControllerMeta);
    rect.on('modified', () => {
      if (!this.callback || this.busy) {
        return;
      }
      this.busy = true;
      const meta = rect.get('data') as ControllerMeta;
      const current = rect.calcTransformMatrix();
      const delta = fabricDeltaToPdfDelta(meta.fold, current, this.pageHeightPt);
      this.callback({ id: meta.id, kind: meta.kind, delta })
        .then(() => {
          rect.set('data', { ...meta, fold: current });
        })
        .finally(() => {
          this.busy = false;
        });
    });
    this.canvas.add(rect);
  }
}

