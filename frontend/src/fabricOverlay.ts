import type { PageIR, PageObject } from './types';
import { fabricDeltaToPdfDelta, Matrix, S } from './coords';

type TransformCallback = (payload: {
  id: string;
  kind: 'text' | 'image';
  pageIndex: number;
  delta: Matrix;
}) => Promise<void>;

type OverlayMeta = {
  id: string;
  kind: 'text' | 'image';
  pageIndex: number;
  F0: Matrix;
};

let cachedFabric: any | null = null;

async function loadFabric(): Promise<any> {
  if (cachedFabric) {
    return cachedFabric;
  }
  const FabricNS = await import('fabric');
  cachedFabric = (FabricNS as any).fabric ?? (FabricNS as any).default ?? FabricNS;
  return cachedFabric;
}

export class FabricOverlayManager {
  private canvas: any = null;
  private fabric: any = null;
  private pageHeightPt = 0;
  private transformCb: TransformCallback | null = null;

  async mount(
    wrapper: HTMLElement,
    size: { width: number; height: number },
    pageHeightPt: number,
    cb: TransformCallback,
  ): Promise<void> {
    this.dispose();
    const fabric = await loadFabric();
    this.fabric = fabric;
    const canvasEl = document.createElement('canvas');
    canvasEl.width = size.width;
    canvasEl.height = size.height;
    canvasEl.className = 'fabric-page-overlay';
    wrapper.innerHTML = '';
    wrapper.appendChild(canvasEl);

    this.canvas = new fabric.Canvas(canvasEl, {
      selection: false,
      preserveObjectStacking: true,
    });
    this.pageHeightPt = pageHeightPt;
    this.transformCb = cb;
  }

  dispose(): void {
    if (this.canvas) {
      this.canvas.dispose();
      this.canvas = null;
      this.fabric = null;
    }
  }

  async populate(page: PageIR): Promise<void> {
    if (!this.canvas || !this.fabric) {
      throw new Error('Overlay canvas not initialised');
    }
    this.canvas.clear();
    for (const object of page.objects) {
      const rect = this.createController(page, object);
      this.canvas.add(rect);
    }
    this.canvas.requestRenderAll();
  }

  private createController(page: PageIR, object: PageObject): any {
    if (!this.fabric) {
      throw new Error('Fabric namespace missing');
    }
    const bbox = object.bbox ?? this.estimateBBox(object);
    const [left, top, width, height] = this.ptRectToPx(page.heightPt, bbox);
    const rect = new this.fabric.Rect({
      left,
      top,
      width,
      height,
      fill: 'rgba(37, 99, 235, 0.08)',
      stroke: '#60a5fa',
      strokeDashArray: [8, 4],
      strokeWidth: 1,
      cornerColor: '#2563eb',
      borderColor: '#2563eb',
      transparentCorners: false,
      selectable: true,
      originX: 'left',
      originY: 'top',
      objectCaching: false,
    });
    const meta: OverlayMeta = {
      id: object.id,
      kind: object.kind,
      pageIndex: page.index,
      F0: rect.calcTransformMatrix() as Matrix,
    };
    rect.set('data', meta);
    rect.on('modified', () => {
      void this.handleTransform(rect);
    });
    return rect;
  }

  private ptRectToPx(pageHeightPt: number, bbox: [number, number, number, number]) {
    const [x0, y0, x1, y1] = bbox;
    const widthPt = x1 - x0;
    const heightPt = y1 - y0;
    const left = x0 / S;
    const top = (pageHeightPt - y1) / S;
    const width = widthPt / S;
    const height = heightPt / S;
    return [left, top, width, height] as const;
  }

  private estimateBBox(object: PageObject): [number, number, number, number] {
    if (object.kind === 'text') {
      const fontSize = object.font.size || 12;
      const width = fontSize * 6;
      const height = fontSize;
      const [a, , , , e, f] = object.Tm;
      const extent = Math.max(Math.abs(a), width);
      return [e, f - height, e + extent, f];
    }
    const [a, , , d, e, f] = object.cm;
    const width = Math.abs(a) || 40;
    const height = Math.abs(d) || 40;
    return [e, f - height, e + width, f];
  }

  private async handleTransform(object: any) {
    if (!this.canvas || !this.transformCb) {
      return;
    }
    const meta = object.get('data') as OverlayMeta | undefined;
    if (!meta) {
      return;
    }
    const matrix = object.calcTransformMatrix() as Matrix;
    try {
      const delta = fabricDeltaToPdfDelta(meta.F0, matrix, this.pageHeightPt);
      await this.transformCb({
        id: meta.id,
        kind: meta.kind,
        pageIndex: meta.pageIndex,
        delta,
      });
      meta.F0 = matrix;
      object.set('data', meta);
      object.setCoords();
      this.canvas.requestRenderAll();
    } catch (error) {
      console.error('Failed to apply transform', error);
      this.applyMatrix(object, meta.F0);
      this.canvas.requestRenderAll();
    }
  }

  private applyMatrix(object: any, matrix: Matrix) {
    if (!this.fabric) {
      return;
    }
    const decomposed = this.fabric.util.qrDecompose(matrix);
    object.set({
      flipX: false,
      flipY: false,
      angle: decomposed.angle,
      scaleX: decomposed.scaleX,
      scaleY: decomposed.scaleY,
      skewX: decomposed.skewX,
      skewY: decomposed.skewY,
      left: decomposed.translateX,
      top: decomposed.translateY,
    });
    object.setCoords();
  }
}
