import type { FabricObject } from 'fabric';
import { fabric } from 'fabric';
import type { PageObject } from './types';
import { pxToPtMatrix, ptToPxMatrix, type Matrix } from './coords';

export type FabricBinding = {
  fabricObject: FabricObject;
  irObject: PageObject;
  baseMatrixPx: Matrix;
};

export function createController(
  obj: PageObject,
  pageHeightPt: number
): FabricBinding {
  const pxMatrix = ptToPxMatrix(pageHeightPt);
  const bbox = obj.bbox;
  const widthPx = (bbox[2] - bbox[0]) / pxMatrix[0];
  const heightPx = (bbox[3] - bbox[1]) / Math.abs(pxMatrix[3]);
  const rect = new fabric.Rect({
    left: bbox[0] * pxMatrix[0] + pxMatrix[4],
    top: pxMatrix[5] + bbox[1] * pxMatrix[2],
    width: Math.max(widthPx, 1),
    height: Math.max(heightPx, 1),
    fill: 'transparent',
    stroke: obj.kind === 'text' ? '#0080ff' : '#ff4081',
    strokeDashArray: obj.kind === 'text' ? [6, 4] : [12, 4],
    strokeWidth: 1,
    selectable: true,
    hasControls: true,
    hasBorders: false
  });

  const baseMatrixPx: Matrix = [
    rect.a || 1,
    rect.b || 0,
    rect.c || 0,
    rect.d || 1,
    rect.left || 0,
    rect.top || 0
  ];

  return { fabricObject: rect, irObject: obj, baseMatrixPx };
}

export function updateControllerMatrix(binding: FabricBinding, pageHeightPt: number) {
  const obj = binding.irObject;
  const pxMatrix = ptToPxMatrix(pageHeightPt);
  const left = obj.bbox[0];
  const bottom = obj.bbox[1];
  const topPx = pxMatrix[5] + bottom * pxMatrix[2];
  binding.fabricObject.set({
    left: left * pxMatrix[0] + pxMatrix[4],
    top: topPx
  });
  binding.fabricObject.setCoords();
}

export function fabricMatrixFromObject(fabricObject: FabricObject): Matrix {
  const { a, b, c, d, left, top } = fabricObject;
  return [a ?? 1, b ?? 0, c ?? 0, d ?? 1, left ?? 0, top ?? 0];
}
