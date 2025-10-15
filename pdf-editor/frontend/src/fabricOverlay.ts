import { fabric } from "fabric";
import type { EditorState, IRObject } from "./types";
import type { PdfPreviewHandle } from "./pdfPreview";
import { fabricDeltaToPdfDelta } from "./coords";
import type { ApiClient } from "./api";

interface OverlayEntry {
  canvas: fabric.Canvas;
  objectIndex: Map<string, fabric.Object>;
}

export function initFabricOverlay(
  state: EditorState,
  preview: PdfPreviewHandle,
  api: ApiClient,
) {
  preview.canvases.forEach((canvasEl, pageIndex) => {
    const overlayCanvas = document.createElement("canvas");
    overlayCanvas.width = canvasEl.width;
    overlayCanvas.height = canvasEl.height;
    overlayCanvas.className = "fabric-overlay";
    canvasEl.parentElement?.appendChild(overlayCanvas);

    const fabricCanvas = new fabric.Canvas(overlayCanvas, {
      selection: false,
    });

    const entry: OverlayEntry = {
      canvas: fabricCanvas,
      objectIndex: new Map(),
    };

    state.fabricOverlays.set(pageIndex, entry);

    const page = state.pages[pageIndex];
    page.objects.forEach((obj) => {
      const controller = createControllerForObject(obj);
      entry.canvas.add(controller);
      entry.objectIndex.set(obj.id, controller);

      controller.on("modified", async () => {
        if (!state.docId) return;
        const base = (controller as any).initialMatrix as number[] | undefined;
        if (!base) return;
        const delta = fabricDeltaToPdfDelta(
          base,
          controller.calcTransformMatrix(),
          page.heightPt,
        );
        await api.patch(state.docId, [
          {
            op: "transform",
            target: { page: pageIndex, id: obj.id },
            kind: obj.kind,
            deltaMatrixPt: delta,
          },
        ]);
      });
    });
  });
}

function createControllerForObject(obj: IRObject): fabric.Object {
  const controller = new fabric.Rect({
    left: obj.bbox[0],
    top: obj.bbox[1],
    width: obj.bbox[2] - obj.bbox[0],
    height: obj.bbox[3] - obj.bbox[1],
    fill: "rgba(0,0,0,0)",
    stroke: obj.kind === "text" ? "#3498db" : "#e67e22",
    strokeWidth: 1,
    hasRotatingPoint: true,
  });
  (controller as any).initialMatrix = controller.calcTransformMatrix();
  return controller;
}
