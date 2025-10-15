import type { FabricObjectMeta } from "./fabricOverlay";
import { pxToPtMatrix, invertMatrix, multiplyMatrix } from "./coords";
import type { DocumentIr, Matrix, PageIr } from "./types";

export function computeFabricMatrix(page: PageIr, matrix: Matrix): Matrix {
  const ptToPx = invertMatrix(pxToPtMatrix(page.heightPt));
  return multiplyMatrix(ptToPx, matrix);
}

export function deriveOverlayMeta(ir: DocumentIr): FabricObjectMeta[] {
  const meta: FabricObjectMeta[] = [];
  for (const page of ir.pages) {
    for (const object of page.objects) {
      const baseMatrix =
        object.kind === "text"
          ? object.Tm
          : object.kind === "image"
          ? object.cm
          : object.cm;
      meta.push({
        pageIndex: page.index,
        objectId: object.id,
        initialMatrix: computeFabricMatrix(page, baseMatrix),
        bbox: object.bbox,
        kind: object.kind,
      });
    }
  }
  return meta;
}
