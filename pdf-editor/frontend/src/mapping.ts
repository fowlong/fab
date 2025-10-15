import type { fabric } from 'fabric';
import type { PageIR, PageObject, TextRun } from './types';
import { bboxToFabricRect, ptToPxMatrix, type Matrix } from './coords';

export type FabricObjectMeta = {
  id: string;
  pageIndex: number;
  baseMatrix: Matrix;
  kind: PageObject['kind'];
};

export type FabricObjectWithMeta = fabric.Object & { __meta?: FabricObjectMeta };

export function getInitialMatrix(page: PageIR): Matrix {
  return ptToPxMatrix(page.heightPt);
}

export function createController(
  fabricNS: typeof fabric,
  page: PageIR,
  obj: PageObject
): FabricObjectWithMeta {
  const rectGeometry = bboxToFabricRect(obj.bbox, page.heightPt);
  const controller = new fabricNS.Rect({
    left: rectGeometry.left,
    top: rectGeometry.top,
    width: rectGeometry.width,
    height: rectGeometry.height,
    fill: 'rgba(0,0,0,0)',
    stroke: obj.kind === 'text' ? '#1976d2' : '#d32f2f',
    strokeWidth: 1,
    strokeDashArray: obj.kind === 'text' ? [4, 3] : [6, 3],
    transparentCorners: false,
    cornerSize: 8,
    lockScalingFlip: true
  }) as FabricObjectWithMeta;

  controller.__meta = {
    id: obj.id,
    pageIndex: page.index,
    baseMatrix: getObjectMatrix(page, obj),
    kind: obj.kind
  };

  return controller;
}

export function getObjectMatrix(page: PageIR, obj: PageObject): Matrix {
  if (obj.kind === 'text') {
    return obj.Tm;
  }
  if ('cm' in obj) {
    return obj.cm;
  }
  return ptToPxMatrix(page.heightPt);
}

export function updateControllerGeometry(
  controller: FabricObjectWithMeta,
  page: PageIR,
  obj: PageObject
) {
  const rectGeometry = bboxToFabricRect(obj.bbox, page.heightPt);
  controller.set({
    left: rectGeometry.left,
    top: rectGeometry.top,
    width: rectGeometry.width,
    height: rectGeometry.height
  });
  controller.__meta = {
    id: obj.id,
    pageIndex: page.index,
    baseMatrix: getObjectMatrix(page, obj),
    kind: obj.kind
  };
  controller.setCoords();
}

export function isText(obj: PageObject): obj is TextRun {
  return obj.kind === 'text';
}
