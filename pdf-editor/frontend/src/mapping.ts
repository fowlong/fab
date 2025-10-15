import type { FabricObjectDescriptor, IrObject } from "./types";
import { pxToPtMatrix, ptToPxMatrix, invert, multiply } from "./coords";

export function irObjectToFabric(ir: IrObject, pageHeightPt: number): FabricObjectDescriptor {
  const [x0, y0, x1, y1] = ir.bbox;
  const width = x1 - x0;
  const height = y1 - y0;
  const matrixPt = ir.transform ?? [1, 0, 0, 1, x0, y0];
  const pxToPt = pxToPtMatrix(pageHeightPt);
  const ptToPx = invert(pxToPt);
  const matrixPx = multiply(multiply(ptToPx, matrixPt as any), pxToPt);

  return {
    id: ir.id,
    width,
    height,
    matrixPx,
  };
}

export function bboxPtToPx(
  bbox: [number, number, number, number],
  pageHeightPt: number,
): [number, number, number, number] {
  const matrix = ptToPxMatrix(pageHeightPt);
  const transformPoint = (x: number, y: number) => {
    const [a, b, c, d, e, f] = matrix;
    return [a * x + c * y + e, b * x + d * y + f] as const;
  };
  const [x0, y0] = transformPoint(bbox[0], bbox[1]);
  const [x1, y1] = transformPoint(bbox[2], bbox[3]);
  return [x0, y0, x1, y1];
}
