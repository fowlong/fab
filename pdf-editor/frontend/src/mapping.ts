import type { IrPage, PageObject } from "./types";
import type { Matrix } from "./coords";
import { pixelToPdfMatrix, invertMatrix, multiplyMatrix } from "./coords";

export interface FabricMappingResult {
  initialMatrix: Matrix;
}

export function toFabricMatrix(page: IrPage, object: PageObject): FabricMappingResult {
  const pxToPt = pixelToPdfMatrix(page.heightPt);
  const ptToPx = invertMatrix(pxToPt);
  let pdfMatrix: Matrix;
  if (object.kind === "text") {
    pdfMatrix = object.Tm;
  } else {
    pdfMatrix = object.cm;
  }
  const initialMatrix = multiplyMatrix(ptToPx, pdfMatrix);
  return { initialMatrix };
}
