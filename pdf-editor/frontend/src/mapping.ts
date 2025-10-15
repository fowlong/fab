import { fabric } from 'fabric';
import type { DocumentIr, Matrix, PatchOperation } from './types';
import { fabricDeltaToPdfDelta } from './coords';

type ObjectMeta = {
  id: string;
  pageIndex: number;
  baseMatrix: Matrix;
};

export function createFabricMapping(canvas: fabric.Canvas, ir: DocumentIr) {
  const metaByObject = new Map<fabric.Object, ObjectMeta>();

  function initialise() {
    canvas.clear();
    metaByObject.clear();

    for (const page of ir.pages) {
      for (const object of page.objects) {
        const width = object.bbox[2] - object.bbox[0];
        const height = object.bbox[3] - object.bbox[1];
        const rect = new fabric.Rect({
          left: object.bbox[0],
          top: page.heightPt - object.bbox[3],
          width,
          height,
          fill: 'rgba(0,0,0,0)',
          stroke: 'rgba(37, 99, 235, 0.8)',
          strokeWidth: 1,
          selectable: true,
          evented: true,
          objectCaching: false
        });
        rect.set({ angle: 0 });
        canvas.add(rect);
        const baseMatrix = rect.calcTransformMatrix() as Matrix;
        metaByObject.set(rect, {
          id: object.id,
          pageIndex: page.index,
          baseMatrix
        });
      }
    }
  }

  function createTransformPatch(target: fabric.Object): PatchOperation[] {
    const meta = metaByObject.get(target);
    if (!meta) {
      return [];
    }
    const currentMatrix = target.calcTransformMatrix() as Matrix;
    const delta = fabricDeltaToPdfDelta(
      meta.baseMatrix,
      currentMatrix,
      ir.pages[meta.pageIndex]?.heightPt ?? 0
    );
    meta.baseMatrix = currentMatrix;
    return [
      {
        op: 'transform',
        target: { page: meta.pageIndex, id: meta.id },
        deltaMatrixPt: delta,
        kind: 'generic'
      }
    ];
  }

  return { initialise, createTransformPatch };
}
