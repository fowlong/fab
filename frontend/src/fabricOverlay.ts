import { fabric } from "fabric";
import type { ApiClient } from "./api";
import type { DocumentIR } from "./types";
import { fabricDeltaToPdfDelta } from "./coords";
import { createFabricObjectFromIr } from "./mapping";
import type { PdfPreview } from "./pdfPreview";

export interface FabricOverlayOptions {
  api: ApiClient;
  docId: string;
  ir: DocumentIR;
  preview: PdfPreview;
  overlayContainer: HTMLElement;
}

type FabricObjectWithMatrix = fabric.Object & { __originalMatrix?: number[] };

export function bootstrapFabricOverlay(options: FabricOverlayOptions) {
  const { ir, preview, overlayContainer, api, docId } = options;
  overlayContainer.innerHTML = "";
  const canvases: Record<number, fabric.Canvas> = {};

  for (const page of ir.pages) {
    const canvasEl = document.createElement("canvas");
    canvasEl.id = `fabric-page-${page.index}`;
    canvasEl.className = "fabric-overlay";
    overlayContainer.appendChild(canvasEl);

    const fabricCanvas = new fabric.Canvas(canvasEl, {
      selection: false,
      preserveObjectStacking: true
    });

    for (const object of page.objects) {
      const fabricObj = createFabricObjectFromIr(object, preview.pageHeightsPt[page.index]) as FabricObjectWithMatrix;
      fabricCanvas.add(fabricObj);
      fabricObj.on("modified", async () => {
        const delta = fabricDeltaToPdfDelta(
          fabricObj.__originalMatrix ?? (fabricObj.calcTransformMatrix() as any),
          fabricObj.calcTransformMatrix() as any,
          preview.pageHeightsPt[page.index]
        );

        await api.patch(docId, [
          {
            op: "transform",
            kind: object.kind,
            target: { page: page.index, id: object.id },
            deltaMatrixPt: delta
          }
        ]);
        fabricObj.__originalMatrix = fabricObj.calcTransformMatrix() as any;
      });
      fabricObj.__originalMatrix = fabricObj.calcTransformMatrix() as any;
    }

    canvases[page.index] = fabricCanvas;
  }

  return canvases;
}
