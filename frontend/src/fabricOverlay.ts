import * as FabricNS from 'fabric';

const fabric: typeof import('fabric')['fabric'] = (FabricNS as any);

import type { Matrix, PageIR, PageObject } from './types';
import { PX_PER_PT, fabricDeltaToPdfDelta, invert, multiply, pxToPtMatrix } from './coords';

export type TransformHandler = (patch: {
  id: string;
  page: number;
  kind: 'text' | 'image';
  delta: Matrix;
}) => Promise<void>;

type ControllerMeta = {
  id: string;
  page: number;
  kind: 'text' | 'image';
  pageHeightPt: number;
  F0: Matrix;
};

type FabricObject = fabric.Object & { meta?: ControllerMeta };

export class FabricOverlay {
  private canvas: fabric.Canvas;
  private pageHeightPt: number;
  private onTransform: TransformHandler;

  constructor(canvas: fabric.Canvas, pageHeightPt: number, onTransform: TransformHandler) {
    this.canvas = canvas;
    this.pageHeightPt = pageHeightPt;
    this.onTransform = onTransform;
    this.canvas.on('object:modified', (event) => {
      const target = event.target as FabricObject | undefined;
      if (!target || !target.meta) return;
      void this.handleModified(target);
    });
  }

  clear() {
    this.canvas.getObjects().forEach((obj) => this.canvas.remove(obj));
    this.canvas.discardActiveObject();
  }

  setObjects(page: PageIR) {
    this.clear();
    page.objects.forEach((object) => this.addObject(page, object));
    this.canvas.renderAll();
  }

  private async handleModified(target: FabricObject) {
    const meta = target.meta;
    if (!meta) return;
    const Fnew = toAffine(target.calcTransformMatrix());
    try {
      const delta = fabricDeltaToPdfDelta(meta.F0, Fnew, meta.pageHeightPt);
      await this.onTransform({ id: meta.id, page: meta.page, kind: meta.kind, delta });
      meta.F0 = Fnew;
    } catch (err) {
      console.error('failed to apply transform', err);
      target.set('transformMatrix', meta.F0);
      target.setCoords();
      this.canvas.renderAll();
    }
  }

  private addObject(page: PageIR, object: PageObject) {
    if (object.kind === 'text') {
      this.addTextController(page, object);
    } else if (object.kind === 'image') {
      this.addImageController(page, object);
    }
  }

  private addTextController(page: PageIR, object: PageObject & { kind: 'text' }) {
    const widthPt = object.bbox[2] - object.bbox[0];
    const heightPt = object.bbox[3] - object.bbox[1];
    const rect = new fabric.Rect({
      width: Math.max(widthPt * PX_PER_PT, 16),
      height: Math.max(heightPt * PX_PER_PT, 16),
      fill: 'rgba(30, 144, 255, 0.12)',
      stroke: 'rgba(30, 144, 255, 0.8)',
      strokeWidth: 1,
      strokeDashArray: [6, 4],
      originX: 'left',
      originY: 'top',
      transparentCorners: false,
      cornerColor: '#1e90ff',
      cornerSize: 8,
    });

    const tmAdjusted = multiply(object.Tm, [1, 0, 0, 1, 0, heightPt]);
    this.applyMatrix(rect, tmAdjusted, page.heightPt);
    this.attachMetadata(rect, page, object.id, 'text');
    this.canvas.add(rect);
  }

  private addImageController(page: PageIR, object: PageObject & { kind: 'image' }) {
    const widthPt = object.bbox[2] - object.bbox[0];
    const heightPt = object.bbox[3] - object.bbox[1];
    const rect = new fabric.Rect({
      width: Math.max(widthPt * PX_PER_PT, 16),
      height: Math.max(heightPt * PX_PER_PT, 16),
      fill: 'rgba(72, 61, 139, 0.12)',
      stroke: 'rgba(72, 61, 139, 0.8)',
      strokeWidth: 1,
      strokeDashArray: [6, 4],
      originX: 'left',
      originY: 'top',
      transparentCorners: false,
      cornerColor: '#483d8b',
      cornerSize: 8,
    });

    const cmAdjusted = multiply(object.cm, [1, 0, 0, 1, 0, heightPt]);
    this.applyMatrix(rect, cmAdjusted, page.heightPt);
    this.attachMetadata(rect, page, object.id, 'image');
    this.canvas.add(rect);
  }

  private applyMatrix(target: fabric.Rect, matrixPt: Matrix, pageHeightPt: number) {
    const ptToPx = invert(pxToPtMatrix(pageHeightPt));
    const matrixPx = multiply(ptToPx, matrixPt);
    target.set({
      transformMatrix: matrixPx,
      left: 0,
      top: 0,
      angle: 0,
      scaleX: 1,
      scaleY: 1,
    });
    target.setCoords();
  }

  private attachMetadata(target: FabricObject, page: PageIR, id: string, kind: 'text' | 'image') {
    const meta: ControllerMeta = {
      id,
      page: page.index,
      kind,
      pageHeightPt: this.pageHeightPt,
      F0: toAffine(target.calcTransformMatrix()),
    };
    target.meta = meta;
    target.set({
      hasBorders: false,
      lockScalingFlip: true,
    });
  }
}

function toAffine(matrix: number[]): Matrix {
  if (matrix.length === 9) {
    return [matrix[0], matrix[1], matrix[3], matrix[4], matrix[6], matrix[7]];
  }
  return [matrix[0], matrix[1], matrix[2], matrix[3], matrix[4], matrix[5]];
}
