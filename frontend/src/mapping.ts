import type { Matrix } from "./coords";
import { multiplyMatrix, pxToPtMatrix, invertMatrix } from "./coords";
import type { PageObject, PageIR } from "./types";

export interface FabricDescriptor {
  id: string;
  pageIndex: number;
  bboxPx: [number, number, number, number];
  matrixPx: Matrix;
  source: PageObject;
}

export function objectToFabricDescriptor(page: PageIR, object: PageObject): FabricDescriptor {
  const ptToPx = invertMatrix(pxToPtMatrix(page.heightPt));
  const matrixPx = multiplyMatrix(ptToPx, object.kind === "text" ? object.Tm : object.cm);
  const [minX, minY, maxX, maxY] = object.bbox;
  const topLeft = multiplyMatrix(ptToPx, [1, 0, 0, 1, minX, minY]);
  const bottomRight = multiplyMatrix(ptToPx, [1, 0, 0, 1, maxX, maxY]);
  const bboxPx: [number, number, number, number] = [
    topLeft[4],
    topLeft[5],
    bottomRight[4],
    bottomRight[5],
  ];
  return {
    id: object.id,
    pageIndex: page.index,
    bboxPx,
    matrixPx,
    source: object,
  };
}
