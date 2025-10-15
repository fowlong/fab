import { fabric } from "fabric";
import type { FabricOverlayOptions } from "./types";
import { fabricDeltaToPdfDelta } from "./coords";
import { toFabricMatrix } from "./mapping";

export function initializeFabricOverlay(options: FabricOverlayOptions) {
  const { pages, preview, api, docId, onStatus } = options;

  preview.pages.forEach((pagePreview, index) => {
    const canvasEl = document.createElement("canvas");
    canvasEl.className = "fabric-overlay";
    canvasEl.width = pagePreview.canvas.width;
    canvasEl.height = pagePreview.canvas.height;
    pagePreview.canvas.parentElement?.appendChild(canvasEl);

    const canvas = new fabric.Canvas(canvasEl, {
      selection: false,
    });

    const page = pages[index];
    page.objects.forEach((obj) => {
      const rect = new fabric.Rect({
        left: obj.bbox[0],
        top: page.heightPt - obj.bbox[3],
        width: obj.bbox[2] - obj.bbox[0],
        height: obj.bbox[3] - obj.bbox[1],
        fill: "rgba(0,0,0,0)",
        stroke: obj.kind === "text" ? "#0066ff" : "#00aa55",
        strokeDashArray: [6, 4],
        strokeWidth: 1,
        hasControls: true,
        objectCaching: false,
      });
      const mapping = toFabricMatrix(page, obj);
      rect.set({ transformMatrix: mapping.initialMatrix as unknown as number[] });
      rect.setCoords();
      rect.set(
        "data",
        {
          id: obj.id,
          page: index,
          kind: obj.kind,
          matrix: rect.calcTransformMatrix(),
        },
      );
      canvas.add(rect);
    });

    canvas.on("object:modified", async (event) => {
      const target = event.target as fabric.Object | undefined;
      if (!target) {
        return;
      }
      const meta = target.get("data") as { id: string; page: number; kind: string; matrix: number[] };
      const delta = fabricDeltaToPdfDelta(meta.matrix, target.calcTransformMatrix(), pages[index].heightPt);
      onStatus?.("Applying transform...");
      await api.patch(docId, [
        {
          op: "transform",
          target: { page: meta.page, id: meta.id },
          kind: meta.kind,
          deltaMatrixPt: delta,
        },
      ]);
      onStatus?.("Transform applied");
      meta.matrix = target.calcTransformMatrix();
    });
  });
}
