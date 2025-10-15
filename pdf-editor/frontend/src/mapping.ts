import { fabric } from 'fabric';
import type { PageIR } from './types';
import { concat, Matrix, POINTS_PER_PX } from './coords';

const PX_PER_PT = 1 / POINTS_PER_PX;
const IDENTITY: Matrix = [1, 0, 0, 1, 0, 0];

export type FabricEntry = {
  id: string;
  kind: 'text' | 'image' | 'path';
  object: fabric.Object;
  originalMatrix: Matrix;
};

export function mapIrObjectToFabric(page: PageIR, _canvas: HTMLCanvasElement): FabricEntry[] {
  const entries: FabricEntry[] = [];

  for (const object of page.objects) {
    const [x1, y1, x2, y2] = object.bbox;
    const rect = new fabric.Rect({
      left: x1 * PX_PER_PT,
      top: (page.heightPt - y2) * PX_PER_PT,
      width: (x2 - x1) * PX_PER_PT,
      height: (y2 - y1) * PX_PER_PT,
      fill: 'rgba(0,0,0,0)',
      stroke: '#007aff',
      strokeWidth: 1,
      selectable: true,
      evented: true,
    });

    const pdfTransform = object.transform ?? IDENTITY;
    const fabricMatrix = concat(IDENTITY, pdfTransform);
    rect.set({
      transformMatrix: fabricMatrix,
      transparentCorners: false,
      cornerColor: '#007aff',
    });

    entries.push({
      id: object.id,
      kind: object.kind,
      object: rect,
      originalMatrix: rect.calcTransformMatrix() as Matrix,
    });
  }

  return entries;
}
