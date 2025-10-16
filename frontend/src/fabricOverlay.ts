import * as FabricNS from 'fabric';
import type { Canvas as FabricCanvas, Object as FabricObject } from 'fabric';

import type { Matrix } from './coords';
import { S, invert, multiply, pxToPtMatrix, fabricDeltaToPdfDelta } from './coords';
import type { PageObject } from './types';

type FabricNamespace = typeof FabricNS;

const moduleLike = FabricNS as unknown as Record<string, unknown>;
const fabricImpl: FabricNamespace =
  (moduleLike['fabric'] as FabricNamespace | undefined) ??
  (moduleLike['default'] as FabricNamespace | undefined) ??
  ((moduleLike as unknown) as FabricNamespace);

type OverlayMeta = {
  id: string;
  kind: 'text' | 'image';
  pdfMatrix: Matrix;
  fabricMatrix: Matrix;
  pageHeightPt: number;
};

type TransformHandler = (id: string, kind: 'text' | 'image', delta: Matrix) => Promise<void>;

export class FabricOverlay {
  private canvas: FabricCanvas | null = null;
  private handler: TransformHandler;
  private pageHeightPt: number;

  constructor(handler: TransformHandler, pageHeightPt: number) {
    this.handler = handler;
    this.pageHeightPt = pageHeightPt;
  }

  dispose() {
    if (this.canvas) {
      this.canvas.dispose();
    }
    this.canvas = null;
  }

  mount(element: HTMLCanvasElement) {
    this.dispose();
    this.canvas = new fabricImpl.Canvas(element, {
      preserveObjectStacking: true,
      selection: true,
    });
    this.canvas.on('object:modified', (event) => {
      const target = event.target as FabricObject | undefined;
      if (!target) return;
      const meta = target.get('data') as OverlayMeta | undefined;
      if (!meta) return;
      const fnew = target.calcTransformMatrix() as Matrix;
      this.applyTransform(target, meta, meta.fabricMatrix, fnew).catch((err) => {
        console.error('Failed to apply transform', err);
        target.set('transformMatrix', meta.fabricMatrix);
        target.setCoords();
        this.canvas?.renderAll();
      });
    });
  }

  private async applyTransform(
    target: fabric.Object,
    meta: OverlayMeta,
    fold: Matrix,
    fnew: Matrix,
  ) {
    const delta = fabricDeltaToPdfDelta(fold, fnew, this.pageHeightPt);
    const currentCanvas = this.canvas;
    await this.handler(meta.id, meta.kind, delta);
    if (!this.canvas || this.canvas !== currentCanvas) {
      return;
    }
    meta.fabricMatrix = fnew;
    meta.pdfMatrix = multiply(delta, meta.pdfMatrix);
    target.set('transformMatrix', fnew);
    target.setCoords();
    this.canvas?.renderAll();
  }

  sync(objects: PageObject[], sizePx: { width: number; height: number }) {
    if (!this.canvas) {
      return;
    }
    this.canvas.clear();
    this.canvas.setDimensions(sizePx);
    objects.forEach((obj) => {
      const rect = this.createController(obj);
      if (rect) {
        this.canvas?.add(rect);
      }
    });
    this.canvas.renderAll();
  }

  private createController(obj: PageObject): FabricObject | null {
    const bbox = obj.bbox;
    const widthPt = bbox[2] - bbox[0];
    const heightPt = bbox[3] - bbox[1];
    const leftPx = bbox[0] / S;
    const topPx = (this.pageHeightPt - bbox[3]) / S;
    const widthPx = widthPt / S;
    const heightPx = heightPt / S;

    const rect = new fabricImpl.Rect({
      left: leftPx,
      top: topPx,
      width: widthPx,
      height: heightPx,
      fill: 'rgba(0,0,0,0)',
      stroke: '#2563eb',
      strokeWidth: 1,
      strokeDashArray: [6, 4],
      selectable: true,
      objectCaching: false,
      transparentCorners: false,
      originX: 'left',
      originY: 'top',
    });

    const pdfMatrix = getPdfMatrix(obj);
    const fabricMatrix = toFabricMatrix(pdfMatrix, this.pageHeightPt);

    rect.set('transformMatrix', fabricMatrix);
    rect.setCoords();

    const meta: OverlayMeta = {
      id: obj.id,
      kind: obj.kind,
      pdfMatrix,
      fabricMatrix,
      pageHeightPt: this.pageHeightPt,
    };
    rect.set('data', meta);
    return rect as FabricObject;
  }
}

function getPdfMatrix(obj: PageObject): Matrix {
  if (obj.kind === 'text') {
    return obj.Tm;
  }
  if (obj.kind === 'image') {
    return obj.cm;
  }
  throw new Error('Unsupported object kind');
}

function toFabricMatrix(matrix: Matrix, pageHeightPt: number): Matrix {
  const ptToPx = invert(pxToPtMatrix(pageHeightPt));
  return multiply(ptToPx, matrix);
}
