import { fabric } from 'fabric';
import type { Matrix, PageIR, PageObject } from './types';

export interface FabricMetadata {
  id: string;
  kind: PageObject['kind'];
  pageIndex: number;
  baseMatrix: Matrix;
}

export function createController(
  canvas: fabric.Canvas,
  page: PageIR,
  object: PageObject,
  pxPerPt: number
): fabric.Object {
  const [x1, y1, x2, y2] = object.bbox;
  const width = (x2 - x1) * pxPerPt;
  const height = (y2 - y1) * pxPerPt;
  const left = x1 * pxPerPt;
  const top = (page.heightPt - y2) * pxPerPt;

  const rect = new fabric.Rect({
    left,
    top,
    width,
    height,
    fill: 'rgba(59,130,246,0.08)',
    stroke: '#1d4ed8',
    strokeWidth: 1,
    strokeDashArray: [6, 4],
    selectable: true,
    evented: true,
    lockScalingFlip: true,
    transparentCorners: false,
    cornerColor: '#1d4ed8'
  });

  const metadata: FabricMetadata = {
    id: object.id,
    kind: object.kind,
    pageIndex: page.index,
    baseMatrix: rect.calcTransformMatrix() as Matrix
  };

  (rect as any).metadata = metadata;
  canvas.add(rect);
  return rect;
}
