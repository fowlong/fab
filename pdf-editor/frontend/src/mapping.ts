import type { Matrix } from './coords';
import { concat, invert, pxToPtMatrix, toCssPx } from './coords';
import type { ImageObject, PageObject, PathObject, TextObject } from './types';

export type OverlayDescriptor = {
  id: string;
  matrixPx: Matrix;
  widthPx: number;
  heightPx: number;
};

function ptToPxMatrix(pageHeightPt: number): Matrix {
  return invert(pxToPtMatrix(pageHeightPt));
}

function baseMatrixForObject(
  object: PageObject,
): Matrix {
  if (object.kind === 'text') {
    return object.Tm;
  }
  if (object.kind === 'image' || object.kind === 'path') {
    return object.cm;
  }
  throw new Error(`Unsupported object kind ${(object as PageObject).kind}`);
}

function sizeFromBbox(obj: PageObject): { widthPt: number; heightPt: number } {
  const [x1, y1, x2, y2] = obj.bbox;
  return { widthPt: Math.max(0, x2 - x1), heightPt: Math.max(0, y2 - y1) };
}

export function mapObjectToOverlay(
  pageHeightPt: number,
  object: PageObject,
): OverlayDescriptor {
  const matrixPt = baseMatrixForObject(object);
  const ptToPx = ptToPxMatrix(pageHeightPt);
  const matrixPx = concat(ptToPx, matrixPt);
  const { widthPt, heightPt } = sizeFromBbox(object);
  return {
    id: object.id,
    matrixPx,
    widthPx: toCssPx(widthPt),
    heightPx: toCssPx(heightPt),
  };
}

export function isTextObject(object: PageObject): object is TextObject {
  return object.kind === 'text';
}

export function isImageObject(object: PageObject): object is ImageObject {
  return object.kind === 'image';
}

export function isPathObject(object: PageObject): object is PathObject {
  return object.kind === 'path';
}
