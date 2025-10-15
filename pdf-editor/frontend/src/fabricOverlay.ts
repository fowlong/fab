import { fabric } from 'fabric';
import type { DocumentIR, PageObject, PatchOp } from './types';
import { fabricDeltaToPdfDelta, type Matrix } from './coords';
import { mapPageObjectsToFabric } from './mapping';

export interface FabricOverlayOptions {
  container: HTMLElement;
  ir: DocumentIR;
  onPatch: (ops: PatchOp[]) => Promise<void>;
}

interface FabricObjectMeta {
  id: string;
  pageIndex: number;
  initialMatrix: Matrix;
  kind: PageObject['kind'];
}

export class FabricOverlay {
  private readonly canvases: fabric.Canvas[] = [];

  private readonly meta = new WeakMap<fabric.Object, FabricObjectMeta>();

  constructor(private readonly options: FabricOverlayOptions) {
    this.setupCanvases();
    this.populateObjects();
  }

  private setupCanvases() {
    this.canvases.length = 0;
    this.options.container.innerHTML = '';

    for (const page of this.options.ir.pages) {
      const canvasEl = document.createElement('canvas');
      canvasEl.className = 'fabric-overlay-canvas';
      canvasEl.width = Math.round(page.widthPt / (72 / 96));
      canvasEl.height = Math.round(page.heightPt / (72 / 96));

      const canvas = new fabric.Canvas(canvasEl, {
        selection: false,
        preserveObjectStacking: true,
      });

      canvas.on('object:modified', async (event) => {
        const target = event.target;
        if (!target) return;
        const meta = this.meta.get(target);
        if (!meta) return;

        const current = target.calcTransformMatrix() as Matrix;
        const delta = fabricDeltaToPdfDelta(meta.initialMatrix, current, this.options.ir.pages[meta.pageIndex].heightPt);
        target.set('transformMatrix', meta.initialMatrix.slice() as unknown as number[]);
        target.setCoords();
        await this.options.onPatch([
          {
            op: 'transform',
            target: { page: meta.pageIndex, id: meta.id },
            deltaMatrixPt: delta,
            kind: meta.kind,
          },
        ]);
      });

      const wrapper = document.createElement('div');
      wrapper.className = 'fabric-page-wrapper';
      wrapper.appendChild(canvasEl);
      this.options.container.appendChild(wrapper);

      this.canvases.push(canvas);
    }
  }

  private populateObjects() {
    this.options.ir.pages.forEach((page, index) => {
      const descriptors = mapPageObjectsToFabric(page);
      const canvas = this.canvases[index];
      descriptors.forEach((descriptor) => {
        const controller = new fabric.Rect({
          left: descriptor.bboxPx.left,
          top: descriptor.bboxPx.top,
          width: descriptor.bboxPx.width,
          height: descriptor.bboxPx.height,
          fill: 'rgba(0,0,0,0)',
          stroke: '#42a5f5',
          strokeWidth: 1,
          selectable: true,
          hasBorders: false,
          hasControls: true,
        });

        controller.set('transformMatrix', descriptor.initialMatrix.slice() as unknown as number[]);

        this.meta.set(controller, {
          id: descriptor.id,
          pageIndex: descriptor.pageIndex,
          initialMatrix: descriptor.initialMatrix,
          kind: descriptor.kind,
        });

        canvas.add(controller);
      });
    });
  }
}
