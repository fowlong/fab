import { fabric } from "fabric";
import type { ApiClient } from "./api";
import type { DocumentIR, PageIR } from "./types";
import { fabricDeltaToPdfDelta } from "./coords";
import { mapObjectToFabric } from "./mapping";

interface LoadedDocument {
  docId: string;
  ir: DocumentIR;
  canvases: fabric.Canvas[];
}

function getOriginalTransform(object: fabric.Object): [number, number, number, number, number, number] {
  const custom = (object as unknown as { __originalTransform?: [number, number, number, number, number, number] }).__originalTransform;
  if (custom) return custom;
  const matrix = object.calcTransformMatrix();
  return [matrix[0], matrix[1], matrix[2], matrix[3], matrix[4], matrix[5]];
}

export function setupFabricOverlay() {
  let current: LoadedDocument | null = null;

  function dispose() {
    current?.canvases.forEach((c) => c.dispose());
    current = null;
  }

  function createCanvasForPage(page: PageIR, host: HTMLCanvasElement) {
    const canvas = new fabric.Canvas(host, {
      selection: false
    });
    canvas.setWidth(host.width);
    canvas.setHeight(host.height);
    return canvas;
  }

  function load(ir: DocumentIR, api: ApiClient, docId: string) {
    dispose();
    const canvases: fabric.Canvas[] = [];

    const stack = document.querySelector<HTMLElement>("#canvasStack");
    if (!stack) {
      throw new Error("Missing canvas stack");
    }

    ir.pages.forEach((page) => {
      const overlayCanvas = document.createElement("canvas");
      overlayCanvas.classList.add("fabric-overlay");
      overlayCanvas.width = page.widthPx;
      overlayCanvas.height = page.heightPx;
      stack.appendChild(overlayCanvas);
      const fabricCanvas = createCanvasForPage(page, overlayCanvas);
      canvases.push(fabricCanvas);

      page.objects.forEach((object) => {
        const fabricObject = mapObjectToFabric(page, object);
        if (!fabricObject) return;
        fabricCanvas.add(fabricObject);

        fabricObject.on("modified", () => {
          const original = getOriginalTransform(fabricObject);
          const currentMatrix = fabricObject.calcTransformMatrix();
          const delta = fabricDeltaToPdfDelta(
            [original[0], original[1], original[2], original[3], original[4], original[5]],
            [currentMatrix[0], currentMatrix[1], currentMatrix[2], currentMatrix[3], currentMatrix[4], currentMatrix[5]],
            page.heightPt
          );

          void api.patch(docId, [
            {
              op: "transform",
              target: { page: page.index, id: object.id },
              deltaMatrixPt: delta,
              kind: object.kind
            }
          ]);
        });
      });
    });

    current = { docId, ir, canvases };
  }

  function currentDocument() {
    return current;
  }

  return { load, currentDocument };
}
