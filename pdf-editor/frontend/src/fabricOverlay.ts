import { fabric } from 'fabric';
import type { PageIr, PatchOperation } from './types';
import { fabricDeltaToPdfDelta } from './coords';
import { objectToFabric } from './mapping';

export interface OverlayConfig {
  canvas: HTMLCanvasElement;
  page: PageIr;
  onPatch: (ops: PatchOperation[]) => Promise<void>;
}

interface ObjectMeta {
  id: string;
  originalMatrix: [number, number, number, number, number, number];
}

function toMatrix(target: fabric.Object): [number, number, number, number, number, number] {
  const m = target.calcTransformMatrix();
  return [m[0], m[1], m[3], m[4], m[6], m[7]];
}

export function setupOverlay({ canvas, page, onPatch }: OverlayConfig): fabric.Canvas {
  const fabricCanvas = new fabric.Canvas(canvas, {
    selection: false,
    preserveObjectStacking: true,
  });

  page.objects.forEach((obj) => {
    const binding = objectToFabric(obj, page.heightPt);
    if (!binding) {
      return;
    }
    binding.object.data = {
      id: obj.id,
      originalMatrix: toMatrix(binding.object),
    } satisfies ObjectMeta;
    fabricCanvas.add(binding.object);
  });

  fabricCanvas.on('object:modified', async (event) => {
    const target = event.target;
    if (!target) {
      return;
    }
    const meta = target.data as ObjectMeta | undefined;
    if (!meta) {
      return;
    }
    const next = toMatrix(target);
    const delta = fabricDeltaToPdfDelta(meta.originalMatrix, next, page.heightPt);
    const op: PatchOperation = {
      op: 'transform',
      target: { page: page.index, id: meta.id },
      deltaMatrixPt: delta,
      kind: page.objects.find((o) => o.id === meta.id)?.kind ?? 'text',
    } as PatchOperation;
    await onPatch([op]);
    meta.originalMatrix = next;
  });

  return fabricCanvas;
}
