import type { FabricObjectMeta } from './fabricOverlay';
import type { IrObject, PageIR } from './types';
import { invert, multiply, pxToPtMatrix } from './coords';

export interface FabricDescriptor {
  id: string;
  kind: IrObject['kind'];
  matrixPx: [number, number, number, number, number, number];
  bboxPx: { left: number; top: number; width: number; height: number };
}

export function irObjectToFabric(
  page: PageIR,
  object: IrObject
): FabricDescriptor {
  const ptToPx = invert(pxToPtMatrix(page.heightPt));
  const [x0, y0, x1, y1] = object.bbox;
  const widthPx = ptToPx[0] * (x1 - x0);
  const heightPx = Math.abs(ptToPx[3]) * (y1 - y0);
  const left = ptToPx[0] * x0 + ptToPx[4];
  const top = ptToPx[3] * y1 + ptToPx[5];

  let matrixPt: [number, number, number, number, number, number];
  if (object.kind === 'text') {
    matrixPt = object.Tm;
  } else {
    matrixPt = object.cm;
  }
  const matrixPx = multiply(ptToPx, matrixPt);

  return {
    id: object.id,
    kind: object.kind,
    matrixPx,
    bboxPx: { left, top, width: widthPx, height: heightPx },
  };
}

export function buildFabricMeta(descriptor: FabricDescriptor): FabricObjectMeta {
  return {
    id: descriptor.id,
    kind: descriptor.kind,
    initialMatrix: descriptor.matrixPx,
  };
}
