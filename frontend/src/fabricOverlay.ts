import * as FabricNS from 'fabric';
const fabric: any =
  (FabricNS as any).fabric ?? (FabricNS as any).default ?? (FabricNS as any);

import type { ImageObject, PageIR, PageObject, TextObject } from './types';
import { fabricDeltaToPdfDelta, S, type Matrix } from './coords';
import type { PatchOperation } from './types';
import { postPatch } from './api';

type ControllerMeta = {
  id: string;
  page: number;
  kind: 'text' | 'image';
  fabricMatrix: Matrix;
};

type MountOptions = {
  docId: string;
  ir: DocumentIR;
  overlayHost: HTMLElement;
  pageCanvasSize: { widthPx: number; heightPx: number };
  pageSizePt: { widthPt: number; heightPt: number };
  onPatched?: () => Promise<void>;
};

const STROKE_STYLE = '#2563eb';

export class FabricOverlayManager {
  private canvas: any | null = null;
  private docId: string | null = null;
  private pageHeightPt = 0;
  private pending = false;

  async mount(options: MountOptions): Promise<void> {
    this.dispose();
    this.docId = options.docId;
    this.pageHeightPt = options.pageSizePt.heightPt;
    options.overlayHost.innerHTML = '';

    const canvasEl = document.createElement('canvas');
    canvasEl.width = options.pageCanvasSize.widthPx;
    canvasEl.height = options.pageCanvasSize.heightPx;
    canvasEl.style.width = `${options.pageCanvasSize.widthPx}px`;
    canvasEl.style.height = `${options.pageCanvasSize.heightPx}px`;
    canvasEl.className = 'fabric-overlay';
    options.overlayHost.appendChild(canvasEl);

    this.canvas = new fabric.Canvas(canvasEl, { selection: true });
    this.populate(options.ir.pages[0], options.onPatched);
  }

  dispose(): void {
    if (this.canvas) {
      this.canvas.dispose();
      this.canvas = null;
    }
    this.docId = null;
    this.pageHeightPt = 0;
  }

  private populate(page: PageIR, onPatched?: () => Promise<void>): void {
    if (!this.canvas) return;
    this.canvas.clear();

    page.objects.forEach((obj) => {
      const controller = this.createController(page, obj);
      if (!controller) return;
      this.canvas!.add(controller);
      const meta: ControllerMeta = {
        id: obj.id,
        page: page.index,
        kind: obj.kind,
        fabricMatrix: toMatrix(controller.calcTransformMatrix()),
      };
      controller.set('data', meta);
      controller.on('modified', () => {
        void this.handleModified(controller, meta, onPatched);
      });
    });
    this.canvas.renderAll();
  }

  private async handleModified(
    object: any,
    meta: ControllerMeta,
    onPatched?: () => Promise<void>,
  ): Promise<void> {
    if (!this.docId || !this.canvas || this.pending) {
      return;
    }
    this.pending = true;
    const previous = meta.fabricMatrix;
    const current = toMatrix(object.calcTransformMatrix());
    try {
      const delta = fabricDeltaToPdfDelta(previous, current, this.pageHeightPt);
      const op: PatchOperation = {
        op: 'transform',
        target: { page: meta.page, id: meta.id },
        deltaMatrixPt: delta,
        kind: meta.kind,
      };
      await postPatch(this.docId, [op]);
      meta.fabricMatrix = current;
      if (onPatched) {
        await onPatched();
      }
    } catch (err) {
      object.set('transformMatrix', previous);
      object.setCoords();
      this.canvas.requestRenderAll();
      console.error('Failed to apply transform', err);
    } finally {
      this.pending = false;
    }
  }

  private createController(page: PageIR, obj: PageObject): any | null {
    const bbox = this.resolveBbox(obj);
    if (!bbox) return null;
    const { centerX, centerY, widthPx, heightPx } = bboxToPixels(bbox, page.heightPt);
    const rect = new fabric.Rect({
      width: widthPx,
      height: heightPx,
      left: centerX,
      top: centerY,
      originX: 'center',
      originY: 'center',
      fill: 'rgba(0,0,0,0)',
      stroke: STROKE_STYLE,
      strokeDashArray: [6, 4],
      strokeWidth: 1,
      selectable: true,
      hasRotatingPoint: true,
    });
    const angle = this.estimateAngle(obj);
    rect.set('angle', angle);
    rect.setControlsVisibility({ mtr: true });
    rect.setCoords();
    return rect;
  }

  private estimateAngle(obj: PageObject): number {
    const matrix = obj.kind === 'text' ? (obj as TextObject).Tm : (obj as ImageObject).cm;
    const angle = Math.atan2(matrix[1], matrix[0]);
    return (angle * 180) / Math.PI;
  }

  private resolveBbox(obj: PageObject): [number, number, number, number] | null {
    if (obj.kind === 'text') {
      const text = obj as TextObject;
      return text.bbox ?? null;
    }
    if (obj.kind === 'image') {
      const image = obj as ImageObject;
      return image.bbox ?? null;
    }
    return null;
  }
}

function toMatrix(values: number[] | Matrix): Matrix {
  const [a, b, c, d, e, f] = values as number[];
  return [a, b, c, d, e, f];
}

function bboxToPixels(bbox: [number, number, number, number], pageHeightPt: number) {
  const [x0, y0, x1, y1] = bbox;
  const widthPt = x1 - x0;
  const heightPt = y1 - y0;
  const widthPx = widthPt / S;
  const heightPx = heightPt / S;
  const leftPx = x0 / S;
  const topPx = (pageHeightPt - y1) / S;
  return {
    centerX: leftPx + widthPx / 2,
    centerY: topPx + heightPx / 2,
    widthPx,
    heightPx,
  };
}
