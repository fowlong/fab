import { fabric } from "fabric";
import type { PageIR, PdfObjectIR } from "./types";

const PX_PER_PT = 96 / 72;

export class EditorMapping {
  private objects = new Map<string, fabric.Object>();

  createController(obj: PdfObjectIR, page: PageIR): fabric.Object {
    const [minX, minY, maxX, maxY] = obj.bbox;
    const widthPt = maxX - minX;
    const heightPt = maxY - minY;

    const widthPx = widthPt * PX_PER_PT;
    const heightPx = heightPt * PX_PER_PT;
    const leftPx = minX * PX_PER_PT;
    const topPx = (page.heightPt - maxY) * PX_PER_PT;

    const rect = new fabric.Rect({
      left: leftPx,
      top: topPx,
      width: widthPx,
      height: heightPx,
      fill: "rgba(0,0,0,0)",
      stroke: "#38bdf8",
      strokeDashArray: [6, 3],
      selectable: true,
      hasBorders: true,
      hasControls: true,
      lockScalingFlip: true,
      originX: "left",
      originY: "top"
    });

    rect.data = { kind: obj.kind };
    (rect as unknown as { irId: string }).irId = obj.id;

    this.objects.set(obj.id, rect);
    return rect;
  }

  getObject(id: string) {
    return this.objects.get(id);
  }
}
