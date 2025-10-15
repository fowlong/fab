import { fabric } from 'fabric';
import type { IrObject, Matrix } from './types';
import { S } from './coords';

export interface FabricBinding {
  object: fabric.Rect;
  matrix: Matrix;
}

const PX_PER_PT = 1 / S;

export function objectToFabric(ir: IrObject, pageHeightPt: number): FabricBinding | null {
  const [minX, minY, maxX, maxY] = ir.bbox ?? [0, 0, 0, 0];
  const widthPt = maxX - minX;
  const heightPt = maxY - minY;
  if (widthPt <= 0 || heightPt <= 0) {
    return null;
  }
  const widthPx = widthPt * PX_PER_PT;
  const heightPx = heightPt * PX_PER_PT;
  const leftPx = minX * PX_PER_PT;
  const topPx = (pageHeightPt - maxY) * PX_PER_PT;

  const rect = new fabric.Rect({
    left: leftPx,
    top: topPx,
    width: widthPx,
    height: heightPx,
    fill: 'rgba(0,0,0,0)',
    stroke: ir.kind === 'text' ? '#2a6cf5' : '#ff6f00',
    strokeWidth: 1,
    strokeDashArray: ir.kind === 'text' ? [4, 2] : [2, 2],
    selectable: true,
    objectCaching: false,
  });

  return {
    object: rect,
    matrix: [1, 0, 0, 1, leftPx, topPx],
  };
}
