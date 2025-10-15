import { fabric } from "fabric";
import type { Matrix } from "./coords";
import { pxToPtMatrix, invert, multiply } from "./coords";
import type { FabricObjectMeta, IrObject } from "./types";

function irMatrix(irObject: IrObject): Matrix {
  if (irObject.kind === "text") {
    return irObject.Tm;
  }
  return irObject.cm;
}

export function createFabricController(irObject: IrObject, pageHeightPt: number) {
  const matrixPt = irMatrix(irObject);
  const ptToPx = invert(pxToPtMatrix(pageHeightPt));
  const matrixPx = multiply(ptToPx, matrixPt);

  const rect = new fabric.Rect({
    left: 0,
    top: 0,
    width: irObject.bbox[2] - irObject.bbox[0],
    height: irObject.bbox[3] - irObject.bbox[1],
    fill: "rgba(0,0,0,0)",
    stroke: "#1971c2",
    strokeWidth: 1,
    selectable: true,
    hasBorders: false,
    hasControls: true,
  });

  const transformMatrix: Matrix = [
    matrixPx[0],
    matrixPx[1],
    matrixPx[2],
    matrixPx[3],
    matrixPx[4],
    matrixPx[5],
  ];

  rect.set({
    transformMatrix,
  });

  const meta: FabricObjectMeta = {
    id: irObject.id,
    page: irObject.page,
    initialMatrix: transformMatrix,
    pageHeightPt,
  };

  return { rect, meta };
}
