import * as FabricNS from 'fabric';
import type { fabric } from 'fabric';

import type { DocumentIR, PageIR, PageObject } from './types';
import { S, fabricDeltaToPdfDelta, invert, multiply, pxToPtMatrix, type Matrix } from './coords';

type FabricNamespace = typeof fabric;
const fabricImpl: FabricNamespace = (FabricNS as unknown as { fabric: FabricNamespace }).fabric ??
  (FabricNS as unknown as { default?: FabricNamespace }).default ??
  (FabricNS as unknown as FabricNamespace);

export type OverlayTransformHandler = (
  payload: OverlayMeta,
  delta: Matrix,
) => Promise<boolean>;

type OverlayMeta = {
  id: string;
  pageIndex: number;
  kind: 'text' | 'image';
  pageHeightPt: number;
  F0: Matrix;
};

type PageSize = {
  width: number;
  height: number;
};

export class FabricOverlayManager {
  private overlays = new Map<number, fabric.Canvas>();

  constructor(private readonly onTransform: OverlayTransformHandler) {}

  reset(): void {
    for (const canvas of this.overlays.values()) {
      canvas.dispose();
    }
    this.overlays.clear();
  }

  mount(ir: DocumentIR, wrappers: HTMLElement[], pageSizes: PageSize[]): void {
    this.reset();
    for (const page of ir.pages) {
      const wrapper = wrappers[page.index];
      const size = pageSizes[page.index];
      if (!wrapper || !size) {
        continue;
      }
      const canvas = this.createCanvas(wrapper, size);
      this.overlays.set(page.index, canvas);
      this.populatePage(canvas, page, size);
    }
  }

  private createCanvas(container: HTMLElement, size: PageSize): fabric.Canvas {
    const canvasEl = document.createElement('canvas');
    canvasEl.width = size.width;
    canvasEl.height = size.height;
    canvasEl.style.width = `${size.width}px`;
    canvasEl.style.height = `${size.height}px`;
    canvasEl.className = 'fabric-page-overlay';
    container.innerHTML = '';
    container.appendChild(canvasEl);
    const canvas = new fabricImpl.Canvas(canvasEl, {
      selection: true,
    });
    canvas.on('object:modified', (event) => {
      const target = event.target as fabric.Object | undefined;
      if (target) {
        void this.handleModified(canvas, target);
      }
    });
    return canvas;
  }

  private populatePage(canvas: fabric.Canvas, page: PageIR, size: PageSize): void {
    page.objects.forEach((object) => {
      const controller = this.createController(object, page);
      if (controller) {
        canvas.add(controller);
      }
    });
    canvas.renderAll();
  }

  private createController(object: PageObject, page: PageIR): fabric.Object | null {
    const bbox = this.getBoundingBox(object);
    const [width, height] = bbox ? [bbox[2] - bbox[0], bbox[3] - bbox[1]] : [page.widthPt * 0.1, page.heightPt * 0.05];
    const widthPx = width / S;
    const heightPx = height / S;
    const rect = new fabricImpl.Rect({
      left: 0,
      top: 0,
      width: widthPx,
      height: heightPx,
      originX: 'left',
      originY: 'top',
      stroke: '#60a5fa',
      strokeWidth: 1,
      fill: 'rgba(59,130,246,0.1)',
      strokeDashArray: [6, 4],
      transparentCorners: false,
      cornerColor: '#1d4ed8',
    });

    const pdfMatrix = this.objectMatrix(object);
    const fabricMatrix = toFabricMatrix(pdfMatrix, page.heightPt);
    (rect as any).transformMatrix = fabricMatrix;
    rect.set('data', {
      id: object.id,
      pageIndex: page.index,
      kind: object.kind,
      pageHeightPt: page.heightPt,
      F0: fabricMatrix,
    } satisfies OverlayMeta);
    rect.setCoords();
    return rect;
  }

  private async handleModified(canvas: fabric.Canvas, object: fabric.Object): Promise<void> {
    const meta = object.get('data') as OverlayMeta | undefined;
    if (!meta) {
      return;
    }
    const newMatrix = toAffineMatrix(object.calcTransformMatrix());
    let delta: Matrix;
    try {
      delta = fabricDeltaToPdfDelta(meta.F0, newMatrix, meta.pageHeightPt);
    } catch (error) {
      console.error('Failed to compute delta matrix', error);
      this.revertTransform(canvas, object, meta.F0);
      return;
    }
    const success = await this.onTransform(meta, delta).catch((error) => {
      console.error('Transform handler failed', error);
      return false;
    });
    if (success) {
      meta.F0 = newMatrix;
      object.set('data', meta);
      object.setCoords();
      canvas.renderAll();
    } else {
      this.revertTransform(canvas, object, meta.F0);
    }
  }

  private revertTransform(canvas: fabric.Canvas, object: fabric.Object, matrix: Matrix): void {
    (object as any).transformMatrix = matrix;
    object.setCoords();
    canvas.renderAll();
  }

  private getBoundingBox(object: PageObject): [number, number, number, number] | undefined {
    if ('bbox' in object && object.bbox) {
      return object.bbox;
    }
    return undefined;
  }

  private objectMatrix(object: PageObject): Matrix {
    if (object.kind === 'text') {
      return object.Tm;
    }
    return object.cm;
  }
}

function toFabricMatrix(matrixPt: Matrix, pageHeightPt: number): Matrix {
  const ptToPx = invert(pxToPtMatrix(pageHeightPt));
  return multiply(ptToPx, matrixPt);
}

function toAffineMatrix(matrix: number[] | Matrix): Matrix {
  if (matrix.length === 6) {
    return matrix as Matrix;
  }
  return [matrix[0], matrix[1], matrix[4], matrix[5], matrix[12], matrix[13]];
}
