import { fabric } from 'fabric';
import { PX_PER_PT, fabricDeltaToPdfDelta } from './coords';
import { attachMeta, getMeta } from './mapping';
import type {
  DocumentIR,
  FabricControllerMeta,
  Matrix,
  PageIR,
  PageObject,
  PatchOperation
} from './types';

type ChangeListener = (ops: PatchOperation[]) => void;

type OverlayContext = {
  canvas: fabric.Canvas;
  meta: FabricControllerMeta;
};

export class FabricOverlayManager {
  private readonly overlays = new Map<number, OverlayContext>();

  constructor(private readonly root: HTMLElement, private readonly onChange: ChangeListener) {}

  mount(ir: DocumentIR): void {
    this.dispose();
    ir.pages.forEach((page) => this.createOverlay(page));
  }

  update(ir: DocumentIR): void {
    ir.pages.forEach((page) => {
      const overlay = this.overlays.get(page.index);
      if (!overlay) {
        this.createOverlay(page);
        return;
      }
      overlay.canvas.getObjects().forEach((obj) => overlay.canvas.remove(obj));
      this.populateOverlay(page, overlay.canvas);
    });
  }

  dispose(): void {
    this.overlays.forEach(({ canvas }) => canvas.dispose());
    this.overlays.clear();
  }

  private createOverlay(page: PageIR): void {
    const canvasEl = document.createElement('canvas');
    canvasEl.id = `fabric-p${page.index}`;
    canvasEl.className = 'fabric-overlay-canvas';
    this.root.appendChild(canvasEl);

    const canvas = new fabric.Canvas(canvasEl, {
      selection: false,
      backgroundColor: 'rgba(0,0,0,0)'
    });
    this.overlays.set(page.index, { canvas, meta: { id: '', pageIndex: page.index, baseMatrixPx: [1, 0, 0, 1, 0, 0] } });
    this.populateOverlay(page, canvas);
  }

  private populateOverlay(page: PageIR, canvas: fabric.Canvas): void {
    page.objects.forEach((object) => {
      const controller = this.createController(page, object);
      canvas.add(controller);
    });
  }

  private createController(page: PageIR, object: PageObject): fabric.Object {
    const [x1, y1, x2, y2] = object.bbox;
    const leftPx = x1 * PX_PER_PT;
    const topPx = (page.heightPt - y2) * PX_PER_PT;
    const widthPx = (x2 - x1) * PX_PER_PT;
    const heightPx = (y2 - y1) * PX_PER_PT;
    const rect = new fabric.Rect({
      left: leftPx,
      top: topPx,
      width: widthPx,
      height: heightPx,
      fill: 'rgba(0,0,0,0)',
      stroke: object.kind === 'text' ? '#0070f3' : '#f39c12',
      strokeWidth: 1,
      selectable: true,
      hasBorders: false,
      hasControls: true,
      transparentCorners: false
    });

    const meta: FabricControllerMeta = {
      id: object.id,
      pageIndex: page.index,
      baseMatrixPx: [1, 0, 0, 1, leftPx, topPx]
    };
    attachMeta(rect, meta);

    rect.on('modified', () => this.handleModification(rect, page.heightPt));
    return rect;
  }

  private handleModification(obj: fabric.Object, pageHeightPt: number): void {
    const meta = getMeta(obj);
    if (!meta) return;
    const prev = meta.baseMatrixPx;
    const current = obj.calcTransformMatrix() as Matrix;
    try {
      const deltaPt = fabricDeltaToPdfDelta(prev, current, pageHeightPt);
      const ops: PatchOperation[] = [
        {
          op: 'transform',
          target: { page: meta.pageIndex, id: meta.id },
          deltaMatrixPt: deltaPt,
          kind: 'text'
        }
      ];
      this.onChange(ops);
      meta.baseMatrixPx = current;
    } catch (err) {
      console.error('Failed to compute transform', err);
    }
  }
}
