import { fabric } from "fabric";
import { SCALE, fabricDeltaToPdfDelta } from "./coords";
import type { RenderedPage } from "./pdfPreview";
import type { BoundingBox, Matrix } from "./types";

export type OverlayKind = "text" | "image" | "path";

export interface FabricObjectMeta {
  pageIndex: number;
  objectId: string;
  initialMatrix: Matrix;
  bbox: BoundingBox;
  kind: OverlayKind;
}

export interface OverlayCallbacks {
  onTransform(target: { page: number; id: string; kind: OverlayKind }, delta: Matrix): void;
  onEditText(target: { page: number; id: string }, text: string): void;
}

interface OverlayObjectData {
  meta: FabricObjectMeta;
  baselineMatrix: Matrix;
}

export class FabricOverlayManager {
  private overlays: Map<number, fabric.Canvas> = new Map();

  constructor(
    pages: RenderedPage[],
    metas: FabricObjectMeta[],
    private callbacks: OverlayCallbacks,
    private pageHeightsPt: Map<number, number>
  ) {
    for (const page of pages) {
      const canvasEl = document.createElement("canvas");
      canvasEl.width = page.canvas.width;
      canvasEl.height = page.canvas.height;
      canvasEl.className = "fabric-overlay";
      canvasEl.style.position = "absolute";
      canvasEl.style.left = "0";
      canvasEl.style.top = "0";
      canvasEl.style.pointerEvents = "none";
      page.container.appendChild(canvasEl);

      const fabricCanvas = new fabric.Canvas(canvasEl, {
        selection: false,
        renderOnAddRemove: true,
      });
      fabricCanvas.setDimensions({ width: canvasEl.width, height: canvasEl.height });
      fabricCanvas.upperCanvasEl.style.pointerEvents = "auto";
      fabricCanvas.upperCanvasEl.style.position = "absolute";
      fabricCanvas.upperCanvasEl.style.left = "0";
      fabricCanvas.upperCanvasEl.style.top = "0";
      this.overlays.set(page.pageIndex, fabricCanvas);
    }

    this.populateObjects(metas);
  }

  private populateObjects(metas: FabricObjectMeta[]) {
    for (const canvas of this.overlays.values()) {
      canvas.clear();
      canvas.off("object:modified");
      canvas.off("mouse:dblclick");
    }

    for (const meta of metas) {
      const canvas = this.overlays.get(meta.pageIndex);
      if (!canvas) continue;
      const [x0, y0, x1, y1] = meta.bbox;
      const widthPx = (x1 - x0) / SCALE;
      const heightPx = (y1 - y0) / SCALE;
      const topPx = canvas.getHeight() - y1 / SCALE;
      const leftPx = x0 / SCALE;
      const rect = new fabric.Rect({
        left: leftPx,
        top: topPx,
        width: Math.max(widthPx, 1),
        height: Math.max(heightPx, 1),
        stroke: meta.kind === "text" ? "#2563eb" : "#10b981",
        strokeWidth: 1,
        fill: "rgba(0,0,0,0)",
        selectable: true,
        hasControls: true,
        objectCaching: false,
      });
      const initialMatrix = toAffineMatrix(rect.calcTransformMatrix());
      (rect as any).data = {
        meta,
        baselineMatrix: initialMatrix,
      } as OverlayObjectData;
      canvas.add(rect);
    }

    for (const [pageIndex, canvas] of this.overlays.entries()) {
      const pageHeightPt = this.pageHeightsPt.get(pageIndex) ?? 0;
      canvas.on("object:modified", (evt) => {
        const target = evt.target as fabric.Object & { data?: OverlayObjectData };
        if (!target || !target.data) return;
        const current = target.calcTransformMatrix();
        const currentMatrix = toAffineMatrix(current);
        const baseline = target.data.baselineMatrix;
        const delta = fabricDeltaToPdfDelta(baseline, currentMatrix, pageHeightPt);
        this.callbacks.onTransform(
          {
            page: target.data.meta.pageIndex,
            id: target.data.meta.objectId,
            kind: target.data.meta.kind,
          },
          delta
        );
        target.data.baselineMatrix = currentMatrix;
      });

      canvas.on("mouse:dblclick", (evt) => {
        const target = (evt.target as fabric.Object & { data?: OverlayObjectData }) || null;
        if (!target || !target.data) return;
        if (target.data.meta.kind !== "text") return;
        const newText = window.prompt("Edit text", "");
        if (newText != null) {
          this.callbacks.onEditText(
            { page: target.data.meta.pageIndex, id: target.data.meta.objectId },
            newText
          );
        }
      });
    }
  }

  public updateMetas(metas: FabricObjectMeta[]) {
    this.populateObjects(metas);
  }

  public dispose() {
    for (const canvas of this.overlays.values()) {
      canvas.dispose();
    }
    this.overlays.clear();
  }
}

function toAffineMatrix(matrix: number[]): Matrix {
  const [a, b, c, d, e, f] = matrix;
  return [a, b, c, d, e, f];
}
