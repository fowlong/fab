import type { Matrix } from "./coords";
import { applyToPoint, ptToPxMatrix } from "./coords";
import type { PageObject } from "./types";

export interface FabricDescriptor {
  id: string;
  kind: PageObject["kind"];
  matrixPx: Matrix;
  bboxPx: { left: number; top: number; width: number; height: number };
}

export function objectToFabric(
  obj: PageObject,
  pageHeightPt: number,
): FabricDescriptor {
  const ptToPx = ptToPxMatrix(pageHeightPt);
  const matrixPt = obj.kind === "text" ? obj.Tm : obj.kind === "image" ? obj.cm : obj.cm;
  const matrixPx: Matrix = [
    ptToPx[0] * matrixPt[0] + ptToPx[2] * matrixPt[1],
    ptToPx[1] * matrixPt[0] + ptToPx[3] * matrixPt[1],
    ptToPx[0] * matrixPt[2] + ptToPx[2] * matrixPt[3],
    ptToPx[1] * matrixPt[2] + ptToPx[3] * matrixPt[3],
    ptToPx[0] * matrixPt[4] + ptToPx[2] * matrixPt[5] + ptToPx[4],
    ptToPx[1] * matrixPt[4] + ptToPx[3] * matrixPt[5] + ptToPx[5],
  ];

  const [x0, y0, x1, y1] = obj.bbox;
  const tl = applyToPoint(ptToPx, x0, y1);
  const br = applyToPoint(ptToPx, x1, y0);
  const bboxPx = {
    left: tl[0],
    top: tl[1],
    width: Math.abs(br[0] - tl[0]),
    height: Math.abs(br[1] - tl[1]),
  };

  return { id: obj.id, kind: obj.kind, matrixPx, bboxPx };
}
