import * as FabricNS from 'fabric';

const fabric: typeof FabricNS.fabric =
  (FabricNS as any).fabric ?? (FabricNS as any).default ?? (FabricNS as any);

import { S, fabricDeltaToPdfDelta, invert, multiply, pxToPtMatrix, type Matrix } from './coords';
import type { DocumentIR, PageIR, PageObject } from './types';

export type TransformHandler = (data: {
  id: string;
  kind: 'text' | 'image';
  delta: Matrix;
}) => Promise<void>;

type OverlayMeta = {
  id: string;
  kind: 'text' | 'image';
  base: Matrix;
};

export class FabricOverlay {
  private canvas: fabric.Canvas;
  private page: PageIR;
  private pageHeightPt: number;
  private onTransform: TransformHandler;

  constructor(canvasElement: HTMLCanvasElement, page: PageIR, handler: TransformHandler) {
    this.page = page;
    this.pageHeightPt = page.heightPt;
    this.onTransform = handler;
    this.canvas = new fabric.Canvas(canvasElement, {
      selection: true,
      preserveObjectStacking: true,
    });
  }

  dispose() {
    this.canvas.dispose();
  }

  populate(ir: DocumentIR) {
    this.canvas.clear();
    const page = ir.pages.find((p) => p.index === this.page.index);
    if (!page) {
      return;
    }
    page.objects.forEach((obj) => {
      const rect = this.createHandle(page, obj);
      this.canvas.add(rect);
    });
    this.canvas.requestRenderAll();
  }

  private createHandle(page: PageIR, obj: PageObject): fabric.Rect {
    const [width, height] = this.sizeToPx(obj.bbox);
    const rect = new fabric.Rect({
      left: 0,
      top: 0,
      width,
      height,
      fill: 'rgba(37,99,235,0.08)',
      stroke: '#2563eb',
      strokeDashArray: [6, 4],
      strokeWidth: 1,
      transparentCorners: false,
      cornerColor: '#1e3a8a',
      originX: 'left',
      originY: 'top',
    });
    const fabricMatrix = this.toFabricMatrix(page, obj);
    rect.transformMatrix = fabricMatrix as unknown as number[];
    rect.set('data', {
      id: obj.id,
      kind: obj.kind,
      base: rect.calcTransformMatrix(),
    } satisfies OverlayMeta);
    rect.on('modified', () => this.handleModification(rect));
    rect.set('dirty', true);
    rect.setCoords();
    return rect;
  }

  private async handleModification(rect: fabric.Rect) {
    const meta = rect.get('data') as OverlayMeta;
    const current = rect.calcTransformMatrix() as Matrix;
    const previous = meta.base as Matrix;
    try {
      const delta = fabricDeltaToPdfDelta(previous, current, this.pageHeightPt);
      await this.onTransform({ id: meta.id, kind: meta.kind, delta });
      meta.base = current;
    } catch (err) {
      rect.transformMatrix = meta.base as unknown as number[];
      rect.setCoords();
      this.canvas.requestRenderAll();
      console.error('Failed to apply transform', err);
    }
  }

  private sizeToPx(bbox: [number, number, number, number]) {
    const [x0, y0, x1, y1] = bbox;
    const widthPt = x1 - x0;
    const heightPt = y1 - y0;
    return [widthPt / S, heightPt / S] as const;
  }

  private toFabricMatrix(page: PageIR, obj: PageObject): Matrix {
    const baseMatrix = obj.kind === 'text' ? obj.Tm : obj.cm;
    const ptToPx = invert(pxToPtMatrix(page.heightPt));
    return multiply(ptToPx, baseMatrix);
  }
}
