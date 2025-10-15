import type { FabricObject } from "./fabricOverlay";
import type { Matrix, PdfObjectIR, PageIR } from "./types";
import { pxToPtMatrix, invert, multiply, applyMatrixToPoint } from "./coords";

export interface FabricDescriptor {
  objectId: string;
  pageIndex: number;
  matrixPx: Matrix;
  bboxPx: { left: number; top: number; width: number; height: number };
  irObject: PdfObjectIR;
}

export function mapIrObjectsToFabric(page: PageIR): FabricDescriptor[] {
  const ptToPx = invert(pxToPtMatrix(page.heightPt));
  const pxScale = 96 / 72;
  return page.objects.map((obj) => {
    const [x0, y0, x1, y1] = obj.bbox;
    const topLeft = applyMatrixToPoint(ptToPx, x0, y1);
    return {
      objectId: obj.id,
      pageIndex: page.index,
      matrixPx: multiply(ptToPx, getObjectMatrix(obj)),
      bboxPx: {
        left: topLeft.x,
        top: topLeft.y,
        width: (x1 - x0) * pxScale,
        height: (y1 - y0) * pxScale,
      },
      irObject: obj,
    };
  });
}

function getObjectMatrix(obj: PdfObjectIR): Matrix {
  if (obj.kind === "text") {
    return obj.Tm;
  }
  return obj.cm;
}

export function updateFabricFromPatch(obj: FabricObject, matrix: Matrix) {
  obj.set({
    a: matrix[0],
    b: matrix[1],
    c: matrix[2],
    d: matrix[3],
    left: matrix[4],
    top: matrix[5],
  });
  obj.setCoords();
}
