import { fabric } from 'fabric';
import { concat, invert, pxToPtMatrix } from './coords';
import type { FabricControllerMeta, Matrix, PageIR, PageObject } from './types';

export function createFabricMatrixFromPdf(
  page: PageIR,
  object: PageObject
): Matrix {
  const base = pxToPtMatrix(page.heightPt);
  const inverse = invert(base as Matrix);
  const pdfMatrix = 'Tm' in object ? object.Tm : 'cm' in object ? object.cm : undefined;
  if (!pdfMatrix) {
    return [1, 0, 0, 1, 0, 0];
  }
  const combined = concat(inverse, pdfMatrix as Matrix);
  return combined;
}

export function attachMeta(target: fabric.Object, meta: FabricControllerMeta): void {
  target.set('data', { ...(target.get('data') as Record<string, unknown>), meta });
}

export function getMeta(target: fabric.Object): FabricControllerMeta | undefined {
  const data = target.get('data') as Record<string, unknown> | undefined;
  return data?.meta as FabricControllerMeta | undefined;
}
