import { fabric } from "fabric";
import type { PageObjectIR } from "./types";

export function mapIrObjectToFabric(object: PageObjectIR, viewport: any) {
  const [x1, y1, x2, y2] = object.bbox;
  const [vx1, vy1] = viewport.convertToViewportPoint(x1, y1);
  const [vx2, vy2] = viewport.convertToViewportPoint(x2, y2);
  const left = Math.min(vx1, vx2);
  const top = Math.min(vy1, vy2);
  const width = Math.abs(vx2 - vx1);
  const height = Math.abs(vy2 - vy1);

  if (!Number.isFinite(width) || !Number.isFinite(height)) {
    return null;
  }

  const controller = new fabric.Rect({
    left,
    top,
    width,
    height,
    fill: "rgba(0,0,0,0)",
    stroke: object.kind === "text" ? "#2563eb" : "#dc2626",
    strokeWidth: 1,
    strokeDashArray: [6, 4],
    selectable: true,
    hasBorders: true,
    hasControls: true,
    objectCaching: false,
  });

  controller.set({ perPixelTargetFind: true });
  return controller;
}
