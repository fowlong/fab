import * as FabricNS from 'fabric';
const fabric: typeof FabricNS =
  (FabricNS as any).fabric ?? (FabricNS as any).default ?? (FabricNS as unknown as typeof FabricNS);

import type { PageIR, PageObject } from './types';
import type { Matrix } from './coords';
import { S, fabricDeltaToPdfDelta } from './coords';

export type TransformHandler = (args: {
  id: string;
  kind: 'text' | 'image';
  delta: Matrix;
}) => Promise<void>;

type ControllerMeta = {
  id: string;
  kind: 'text' | 'image';
  pageHeightPt: number;
  F0: Matrix;
};

export class FabricOverlay {
  private canvas: fabric.Canvas;
  private handler: TransformHandler;
  private pageHeightPt: number;

  constructor(canvasEl: HTMLCanvasElement, pageHeightPt: number, handler: TransformHandler) {
    this.canvas = new fabric.Canvas(canvasEl, { selection: false, preserveObjectStacking: true });
    this.pageHeightPt = pageHeightPt;
    this.handler = handler;
  }

  clear() {
    this.canvas.getObjects().forEach((obj) => this.canvas.remove(obj));
    this.canvas.discardActiveObject();
    this.canvas.requestRenderAll();
  }

  render(page: PageIR) {
    this.clear();
    page.objects.forEach((obj) => this.addController(obj, page));
    this.canvas.requestRenderAll();
  }

  private addController(obj: PageObject, page: PageIR) {
    const bbox = obj.bbox;
    const widthPx = (bbox[2] - bbox[0]) / S;
    const heightPx = (bbox[3] - bbox[1]) / S;
    const leftPx = bbox[0] / S;
    const topPx = (page.heightPt - bbox[3]) / S;

    const rect = new fabric.Rect({
      left: leftPx,
      top: topPx,
      width: Math.max(widthPx, 4),
      height: Math.max(heightPx, 4),
      fill: 'rgba(0,0,0,0)',
      stroke: 'rgba(33, 150, 243, 0.6)',
      strokeDashArray: [6, 4],
      strokeWidth: 1.5,
      transparentCorners: false,
      cornerColor: '#2196F3',
      hasBorders: true,
    });

    const meta: ControllerMeta = {
      id: obj.id,
      kind: obj.kind,
      pageHeightPt: page.heightPt,
      F0: rect.calcTransformMatrix() as Matrix,
    };

    (rect as any).meta = meta;

    rect.on('modified', () => {
      const currentMeta = (rect as any).meta as ControllerMeta | undefined;
      if (!currentMeta) return;
      const Fnew = rect.calcTransformMatrix() as Matrix;
      const delta = fabricDeltaToPdfDelta(currentMeta.F0, Fnew, currentMeta.pageHeightPt);
      this.handler({ id: currentMeta.id, kind: currentMeta.kind, delta })
        .then(() => {
          currentMeta.F0 = Fnew;
        })
        .catch((err) => {
          console.error('patch failed', err);
          rect.set('transformMatrix', currentMeta.F0.slice() as any);
          rect.setCoords();
          this.canvas.requestRenderAll();
        });
    });

    this.canvas.add(rect);
  }
}
