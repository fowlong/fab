import * as FabricNS from 'fabric';

const fabric: typeof FabricNS.fabric =
  (FabricNS as any).fabric ?? (FabricNS as any).default ?? (FabricNS as any);

import type { DocumentIR, ImageObject, PageIR, PageObject, TextObject } from './types';
import { fabricDeltaToPdfDelta, multiply, ptToPxMatrix, S, type Matrix } from './coords';

export type TransformRequest = {
  id: string;
  kind: 'text' | 'image';
  pageIndex: number;
  deltaMatrix: Matrix;
  object: fabric.Object;
};

export type TransformHandler = (request: TransformRequest) => Promise<boolean>;

type OverlayEntry = {
  canvas: fabric.Canvas;
  pageHeightPt: number;
};

type OverlayMeta = {
  id: string;
  kind: 'text' | 'image';
  pageIndex: number;
  baseMatrix: Matrix;
  pageHeightPt: number;
};

export class FabricOverlayManager {
  private overlays = new Map<number, OverlayEntry>();

  constructor(private readonly onTransform: TransformHandler) {}

  reset(): void {
    for (const entry of this.overlays.values()) {
      const container = entry.canvas.lowerCanvasEl.parentElement;
      entry.canvas.dispose();
      container?.remove();
    }
    this.overlays.clear();
  }

  populate(
    ir: DocumentIR,
    wrappers: HTMLElement[],
    pageSizesPx: Array<{ width: number; height: number }>,
  ): void {
    this.reset();
    ir.pages.forEach((page) => {
      const wrapper = wrappers[page.index];
      const size = pageSizesPx[page.index];
      if (!wrapper || !size) {
        return;
      }
      const entry = this.mountOverlay(page, wrapper, size);
      this.attachEvents(entry);
      page.objects.forEach((obj) => this.addObject(entry, page, obj));
      entry.canvas.requestRenderAll();
    });
  }

  private mountOverlay(page: PageIR, wrapper: HTMLElement, size: { width: number; height: number }): OverlayEntry {
    const canvasEl = document.createElement('canvas');
    canvasEl.width = size.width;
    canvasEl.height = size.height;
    canvasEl.style.width = `${size.width}px`;
    canvasEl.style.height = `${size.height}px`;
    canvasEl.className = 'fabric-page-overlay';
    wrapper.appendChild(canvasEl);

    const canvas = new fabric.Canvas(canvasEl, {
      selection: true,
      preserveObjectStacking: true,
    });

    const entry: OverlayEntry = {
      canvas,
      pageHeightPt: page.heightPt,
    };
    this.overlays.set(page.index, entry);
    return entry;
  }

  private attachEvents(entry: OverlayEntry): void {
    entry.canvas.on('object:modified', (evt) => {
      const target = evt.target;
      if (!target) {
        return;
      }
      const meta = target.get('data') as OverlayMeta | undefined;
      if (!meta) {
        return;
      }
      const fold = meta.baseMatrix;
      const fnew = extractMatrix(target);
      const deltaMatrix = fabricDeltaToPdfDelta(fold, fnew, meta.pageHeightPt);
      Promise.resolve(
        this.onTransform({
          id: meta.id,
          kind: meta.kind,
          pageIndex: meta.pageIndex,
          deltaMatrix,
          object: target,
        }),
      )
        .then((ok) => {
          if (ok) {
            meta.baseMatrix = fnew;
          } else {
            applyMatrix(target, fold);
            target.setCoords();
            entry.canvas.requestRenderAll();
          }
        })
        .catch((error) => {
          console.error('transform handler failed', error);
          applyMatrix(target, fold);
          target.setCoords();
          entry.canvas.requestRenderAll();
        });
    });
  }

  private addObject(entry: OverlayEntry, page: PageIR, object: PageObject): void {
    const bbox = object.bbox;
    const widthPx = (bbox[2] - bbox[0]) / S;
    const heightPx = (bbox[3] - bbox[1]) / S;
    const rect = new fabric.Rect({
      left: 0,
      top: 0,
      width: Math.max(widthPx, 4),
      height: Math.max(heightPx, 4),
      fill: 'rgba(0,0,0,0)',
      stroke: '#2563eb',
      strokeDashArray: [6, 4],
      strokeWidth: 1,
      selectable: true,
      objectCaching: false,
      transparentCorners: false,
      hasBorders: true,
    });

    rect.set('originX', 'left');
    rect.set('originY', 'top');

    const pdfMatrix = object.kind === 'text' ? (object as TextObject).Tm : (object as ImageObject).cm;
    const fabricMatrix = multiply(ptToPxMatrix(page.heightPt), pdfMatrix as Matrix);
    applyMatrix(rect, fabricMatrix);

    const meta: OverlayMeta = {
      id: object.id,
      kind: object.kind,
      pageIndex: page.index,
      baseMatrix: extractMatrix(rect),
      pageHeightPt: entry.pageHeightPt,
    };
    rect.set('data', meta);

    entry.canvas.add(rect);
    rect.setCoords();
  }
}

function extractMatrix(target: fabric.Object): Matrix {
  const raw = target.calcTransformMatrix();
  if (Array.isArray(raw) && raw.length === 6) {
    return raw as Matrix;
  }
  if (Array.isArray(raw) && raw.length === 16) {
    return [raw[0], raw[1], raw[4], raw[5], raw[12], raw[13]];
  }
  throw new Error('Unsupported transform matrix shape');
}

function applyMatrix(target: fabric.Object, matrix: Matrix): void {
  target.set('transformMatrix', matrix.slice() as Matrix);
  target.set('left', 0);
  target.set('top', 0);
}
