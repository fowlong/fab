import { fabric } from "fabric";
import type { DocumentIR } from "./types";
import type { ApiClient } from "./api";
import { objectToFabricDescriptor } from "./mapping";
import { fabricDeltaToPdfDelta } from "./coords";

export interface FabricOverlayOptions {
  canvas: HTMLCanvasElement;
  document: DocumentIR;
  api: ApiClient;
}

export function initialiseFabricOverlay({ canvas, document, api }: FabricOverlayOptions): void {
  const firstPage = document.pages[0];
  if (firstPage) {
    const widthPx = Math.round((firstPage.widthPt / 72) * 96);
    const heightPx = Math.round((firstPage.heightPt / 72) * 96);
    canvas.width = widthPx;
    canvas.height = heightPx;
    canvas.style.width = `${widthPx}px`;
    canvas.style.height = `${heightPx}px`;
  }

  const fabricCanvas = new fabric.Canvas(canvas, {
    selection: false,
    backgroundColor: "rgba(0,0,0,0)",
  });

  document.pages.forEach((page) => {
    page.objects.forEach((object) => {
      const descriptor = objectToFabricDescriptor(page, object);
      const width = Math.abs(descriptor.bboxPx[2] - descriptor.bboxPx[0]);
      const height = Math.abs(descriptor.bboxPx[3] - descriptor.bboxPx[1]);
      const rect = new fabric.Rect({
        left: descriptor.bboxPx[0],
        top: descriptor.bboxPx[1],
        width: width || 40,
        height: height || 20,
        fill: "rgba(59,130,246,0.1)",
        stroke: "rgba(37,99,235,0.9)",
        strokeWidth: 1,
        selectable: true,
        hasBorders: true,
        hasControls: true,
      });
      // Store metadata for later
      rect.set("data", {
        id: descriptor.id,
        pageIndex: descriptor.pageIndex,
        matrixPx: descriptor.matrixPx,
        pageHeightPt: page.heightPt,
        kind: object.kind,
      });
      fabricCanvas.add(rect);
    });
  });

  fabricCanvas.on("object:modified", async (evt) => {
    const target = evt.target as fabric.Object & {
      data?: {
        id: string;
        pageIndex: number;
        matrixPx: [number, number, number, number, number, number];
        pageHeightPt: number;
        kind: string;
      };
    };
    if (!target || !target.data) {
      return;
    }
    const { matrixPx, pageHeightPt, id, pageIndex, kind } = target.data;
    const newMatrix = target.calcTransformMatrix() as [number, number, number, number, number, number];
    const delta = fabricDeltaToPdfDelta(matrixPx, newMatrix, pageHeightPt);
    target.set("data", { ...target.data, matrixPx: newMatrix });
    try {
      await api.applyPatch([
        {
          op: "transform",
          target: { page: pageIndex, id },
          deltaMatrixPt: delta,
          kind: kind as never,
        },
      ]);
    } catch (err) {
      console.error("Failed to send transform patch", err);
    }
  });
}
