import { Matrix, pxToPtMatrix } from "./coords";
import { IrObject, PageIR } from "./types";

export interface FabricMapping {
  id: string;
  pageIndex: number;
  initialMatrixPx: Matrix;
}

export function computeInitialMatrix(obj: IrObject, page: PageIR): Matrix {
  const px = pxToPtMatrix(page.heightPt);
  return [px[0], px[1], px[2], px[3], obj.bbox[0], obj.bbox[1]];
}
