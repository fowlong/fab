import { fabric } from 'fabric';
import type { PageIR, PatchOp } from './types';
import type { OverlayObjectMeta } from './mapping';
import { applyMetaToObject, createOverlayObject, extractMeta } from './mapping';
import { fabricDeltaToPdfDelta, Matrix } from './coords';

export type FabricOverlayOptions = {
  canvas: HTMLCanvasElement;
  page: PageIR;
  onPatchRequested: (ops: PatchOp[]) => void;
};

export class FabricOverlay {
  private canvas: fabric.Canvas;
  private options: FabricOverlayOptions;

  constructor(options: FabricOverlayOptions) {
    this.options = options;
    this.canvas = new fabric.Canvas(options.canvas, {
      selection: false,
      preserveObjectStacking: true
    });
    this.populate();
    this.attachHandlers();
  }

  private populate() {
    const { page } = this.options;
    for (const object of page.objects) {
      const { fabricObject, meta } = createOverlayObject(page, object);
      applyMetaToObject(fabricObject, meta);
      this.canvas.add(fabricObject);
    }
  }

  private attachHandlers() {
    this.canvas.on('object:modified', (event) => {
      const target = event.target;
      if (!target) return;
      const meta = extractMeta(target) as OverlayObjectMeta | undefined;
      if (!meta) return;

      const originalMatrix = meta.baseMatrix;
      const currentMatrix = (target.calcTransformMatrix() as Matrix) ?? [1, 0, 0, 1, 0, 0];
      const delta = fabricDeltaToPdfDelta(
        originalMatrix,
        currentMatrix,
        this.options.page.heightPt
      );

      const patch: PatchOp = {
        op: 'transform',
        target: { page: meta.pageIndex, id: meta.id },
        deltaMatrixPt: delta,
        kind: meta.id.startsWith('t:') ? 'text' : meta.id.startsWith('img:') ? 'image' : 'path'
      };
      this.options.onPatchRequested([patch]);
    });
  }

  public dispose() {
    this.canvas.dispose();
  }
}
