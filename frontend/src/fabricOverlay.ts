import { fabric } from 'fabric';
import type { DocumentIR, PageObject, PatchOperation } from './types';
import { fabricDeltaToPdfDelta } from './coords';
import { postPatch } from './api';

export function initialiseFabricOverlay(
  container: HTMLElement,
  ir: DocumentIR,
  docId: string
) {
  for (const page of ir.pages) {
    const canvasEl = container.querySelector(
      `#fabric-p${page.index}`
    ) as HTMLCanvasElement | null;
    if (!canvasEl) {
      continue;
    }

    const canvas = new fabric.Canvas(canvasEl, {
      selection: false
    });

    canvas.add(...page.objects.map((obj) => makeController(obj, page.heightPt)));

    canvas.on('object:modified', async (event) => {
      const target = event.target as fabric.Object & {
        irObject?: PageObject;
        initialMatrix?: [number, number, number, number, number, number];
      };
      if (!target || !target.irObject || !target.initialMatrix) {
        return;
      }

      const fabricMatrix = target.calcTransformMatrix() as unknown as [
        number,
        number,
        number,
        number,
        number,
        number
      ];
      const delta = fabricDeltaToPdfDelta(
        target.initialMatrix,
        fabricMatrix,
        page.heightPt
      );

      const patch: PatchOperation = {
        op: 'transform',
        target: { page: page.index, id: target.irObject.id },
        deltaMatrixPt: delta,
        kind: target.irObject.kind
      } as PatchOperation;

      await postPatch(docId, [patch]);

      target.initialMatrix = fabricMatrix;
    });
  }
}

function makeController(object: PageObject, pageHeightPt: number) {
  const bbox = object.bbox;
  const scale = 1 / (72 / 96);
  const left = bbox[0] * scale;
  const top = (pageHeightPt - bbox[3]) * scale;
  const width = (bbox[2] - bbox[0]) * scale;
  const height = (bbox[3] - bbox[1]) * scale;
  const rect = new fabric.Rect({
    left,
    top,
    width,
    height,
    fill: 'rgba(0,0,0,0)',
    stroke: '#1e88e5',
    strokeWidth: 1,
    selectable: true,
    evented: true
  });
  (rect as any).irObject = object;
  const matrix = rect.calcTransformMatrix();
  (rect as any).initialMatrix = matrix as unknown as [
    number,
    number,
    number,
    number,
    number,
    number
  ];
  return rect;
}
