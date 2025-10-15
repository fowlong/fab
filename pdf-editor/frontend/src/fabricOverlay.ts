import { fabric } from "fabric";
import { getPreviewHandle } from "./pdfPreview";
import { fabricDeltaToPdfDelta } from "./coords";
import type { Matrix } from "./coords";
import type { FabricObjectMeta } from "./types";

const overlayPerPage = new Map<number, fabric.Canvas>();
const metadata = new WeakMap<fabric.Object, FabricObjectMeta>();

export function initialiseFabricOverlay(editor: HTMLElement) {
  const canvas = document.createElement("canvas");
  canvas.id = "fabric-overlay-placeholder";
  canvas.width = 800;
  canvas.height = 600;
  editor.appendChild(canvas);

  const fabricCanvas = new fabric.Canvas(canvas, {
    selection: false,
    preserveObjectStacking: true,
  });

  overlayPerPage.set(0, fabricCanvas);

  fabricCanvas.on("object:modified", (event) => {
    const obj = event.target;
    if (!obj) return;
    const meta = metadata.get(obj);
    const preview = getPreviewHandle();
    if (!meta || !preview) return;
    const currentMatrix = obj.calcTransformMatrix();
    const delta = fabricDeltaToPdfDelta(
      meta.initialMatrix,
      currentMatrix,
      meta.pageHeightPt,
    );
    meta.initialMatrix = [
      currentMatrix[0] ?? 1,
      currentMatrix[1] ?? 0,
      currentMatrix[2] ?? 0,
      currentMatrix[3] ?? 1,
      currentMatrix[4] ?? 0,
      currentMatrix[5] ?? 0,
    ] as Matrix;
    console.debug("PDF delta", meta.id, delta);
  });
}

export function registerObject(
  pageIndex: number,
  obj: fabric.Object,
  meta: FabricObjectMeta,
) {
  const canvas = overlayPerPage.get(pageIndex);
  if (!canvas) {
    throw new Error(`Canvas for page ${pageIndex} missing`);
  }
  metadata.set(obj, meta);
  canvas.add(obj);
}
