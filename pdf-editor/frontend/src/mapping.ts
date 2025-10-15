import { fabric } from "fabric";
import { concat, invert, pxToPtMatrix } from "./coords";
import type { IrObject, PdfMatrix } from "./types";

export interface OverlayDescriptor {
  ir: IrObject;
  pageIndex: number;
  fabricObject: fabric.Rect;
  baseMatrix: PdfMatrix;
}

export function matrixToFabricTransform(
  m: PdfMatrix,
  pageHeightPt: number
): PdfMatrix {
  const ptToPx = invert(pxToPtMatrix(pageHeightPt));
  return concat(ptToPx, m);
}

export function bboxToFabricRect(
  obj: IrObject,
  pageHeightPt: number
): fabric.Rect {
  const [x1, y1, x2, y2] = obj.bbox;
  const ptToPx = invert(pxToPtMatrix(pageHeightPt));
  const topLeft = transformPoint(ptToPx, x1, y2);
  const bottomRight = transformPoint(ptToPx, x2, y1);
  const rect = new fabric.Rect({
    left: topLeft.x,
    top: topLeft.y,
    width: bottomRight.x - topLeft.x,
    height: bottomRight.y - topLeft.y,
    fill: "rgba(0,0,0,0)",
    stroke: obj.kind === "text" ? "#2563eb" : "#dc2626",
    strokeWidth: 1.5,
    selectable: true,
    hasControls: true,
    hasBorders: true
  });
  rect.set("objectCaching", false);
  rect.set("hoverCursor", "move");
  return rect;
}

function transformPoint(m: PdfMatrix, x: number, y: number) {
  const [a, b, c, d, e, f] = m;
  return {
    x: a * x + c * y + e,
    y: b * x + d * y + f
  };
}
