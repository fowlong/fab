import { fabric } from "fabric";
import { fabricDeltaToPdfDelta } from "./coords";
import { bboxPtToPx } from "./mapping";
import type { ApiClient } from "./api";
import type { EditorState, FabricOverlayHandle, IrPage } from "./types";

interface InitOptions {
  page: IrPage;
  container: HTMLElement;
  pdfCanvas: HTMLCanvasElement;
  api: ApiClient;
  state: EditorState;
}

export function initialiseFabricOverlay(options: InitOptions): FabricOverlayHandle {
  const { page, container, pdfCanvas } = options;
  const originalMatrices = new WeakMap<fabric.Object, number[]>();
  const overlayCanvas = document.createElement("canvas");
  overlayCanvas.width = pdfCanvas.width;
  overlayCanvas.height = pdfCanvas.height;
  overlayCanvas.style.position = "absolute";
  overlayCanvas.style.left = "0";
  overlayCanvas.style.top = "0";
  container.appendChild(overlayCanvas);

  const canvas = new fabric.Canvas(overlayCanvas, {
    selection: false,
    backgroundColor: "rgba(0,0,0,0)",
  });

  canvas.on("object:modified", async (event) => {
    const obj = event.target as (fabric.Object & { irId?: string }) | undefined;
    if (!obj || !obj.irId) return;

    const originalMatrix = originalMatrices.get(obj);
    const currentMatrix = obj.calcTransformMatrix();
    if (!originalMatrix) return;

    const delta = fabricDeltaToPdfDelta(originalMatrix, currentMatrix, page.heightPt);
    console.debug("TODO send transform patch", obj.irId, delta);
  });

  const handle: FabricOverlayHandle = {
    canvas,
    pageIndex: page.index,
    rebuild(objects) {
      canvas.clear();
      for (const object of objects) {
        const [x0, y0, x1, y1] = bboxPtToPx(object.bbox, page.heightPt);
        const rect = new fabric.Rect({
          left: x0,
          top: y0,
          width: x1 - x0,
          height: y1 - y0,
          fill: "rgba(0,0,0,0)",
          stroke: "#22d3ee",
          strokeWidth: 2,
          selectable: true,
          hasBorders: false,
          hasControls: true,
        });
        (rect as fabric.Object & { irId: string }).irId = object.id;
        originalMatrices.set(rect, rect.calcTransformMatrix());
        canvas.add(rect);
      }
      canvas.requestRenderAll();
    },
  };

  handle.rebuild(page.objects);

  return handle;
}
