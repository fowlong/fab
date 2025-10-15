import { fabric } from 'fabric';
import type { DocumentIR, PatchOperation } from './types';
import { fabricDeltaToPdfDelta, Matrix } from './coords';
import { mapIrObjectToFabric } from './mapping';

type OverlayOptions = {
  onPatch: (ops: PatchOperation[]) => Promise<unknown>;
};

export function initialiseFabricOverlay(ir: DocumentIR, container: HTMLElement, options: OverlayOptions) {
  for (const pageIR of ir.pages) {
    const wrapper = container.querySelector<HTMLDivElement>(`.page-wrapper:nth-child(${pageIR.index + 1})`);
    if (!wrapper) continue;

    const width = wrapper.clientWidth || parseFloat(wrapper.style.getPropertyValue('--page-width'));
    const height = wrapper.clientHeight || parseFloat(wrapper.style.getPropertyValue('--page-height'));

    const canvasEl = document.createElement('canvas');
    canvasEl.id = `fabric-p${pageIR.index}`;
    canvasEl.className = 'fabric-overlay';
    canvasEl.width = width;
    canvasEl.height = height;
    wrapper.appendChild(canvasEl);

    const canvas = new fabric.Canvas(canvasEl, {
      selection: true,
      preserveObjectStacking: true,
    });

    const fabricObjects = mapIrObjectToFabric(pageIR, canvasEl);

    fabricObjects.forEach((entry) => {
      canvas.add(entry.object);
      entry.object.on('modified', async () => {
        const current = entry.object.calcTransformMatrix() as Matrix;
        const delta = fabricDeltaToPdfDelta(entry.originalMatrix, current, pageIR.heightPt);
        const op: PatchOperation = {
          op: 'transform',
          kind: entry.kind,
          target: { page: pageIR.index, id: entry.id },
          deltaMatrixPt: [...delta] as Matrix,
        };
        await options.onPatch([op]);
        entry.originalMatrix = current;
      });
    });

    canvas.renderAll();
  }
}
