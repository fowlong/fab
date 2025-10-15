import { fabric } from "fabric";
import type { DocumentIR, Matrix, PatchOp, PdfObjectIR } from "./types";
import { fabricDeltaToPdfDelta } from "./coords";
import { mapIrObjectsToFabric, type FabricDescriptor } from "./mapping";
import { sendPatch } from "./api";

export type FabricObject = fabric.Object & {
  metadata?: FabricMetadata;
};

export interface FabricMetadata {
  id: string;
  pageIndex: number;
  baseMatrix: Matrix;
}

export function initialiseFabricOverlay(
  container: HTMLElement,
  ir: DocumentIR,
  getDocId: () => string | null,
) {
  for (const page of ir.pages) {
    const pageWrapper = container.querySelector<HTMLDivElement>(`.page:nth-child(${page.index + 1})`);
    if (!pageWrapper) continue;

    const overlayCanvas = document.createElement("canvas");
    overlayCanvas.id = `fabric-p${page.index}`;
    overlayCanvas.width = pageWrapper.clientWidth;
    overlayCanvas.height = pageWrapper.clientHeight;
    overlayCanvas.className = "fabric-overlay";
    pageWrapper.append(overlayCanvas);

    const canvas = new fabric.Canvas(overlayCanvas, {
      selection: false,
    });

    const descriptors = mapIrObjectsToFabric(page);
    descriptors.forEach((descriptor) => createController(canvas, descriptor));

    canvas.on("object:modified", async (event) => {
      const target = event.target as FabricObject | undefined;
      if (!target || !target.metadata) return;
      const docId = getDocId();
      if (!docId) return;

      const nextMatrix: Matrix = [
        target.a ?? 1,
        target.b ?? 0,
        target.c ?? 0,
        target.d ?? 1,
        target.left ?? 0,
        target.top ?? 0,
      ];
      const delta = fabricDeltaToPdfDelta(
        target.metadata.baseMatrix,
        nextMatrix,
        ir.pages[target.metadata.pageIndex].heightPt,
      );

      const patch: PatchOp = {
        op: "transform",
        target: { page: target.metadata.pageIndex, id: target.metadata.id },
        deltaMatrixPt: delta,
        kind: inferPatchKind(target.metadata.id, ir.pages[target.metadata.pageIndex].objects),
      } as PatchOp;

      try {
        await sendPatch(docId, [patch]);
        target.metadata.baseMatrix = nextMatrix;
      } catch (error) {
        console.error("Failed to apply patch", error);
      }
    });
  }
}

function createController(canvas: fabric.Canvas, descriptor: FabricDescriptor) {
  const controller = new fabric.Rect({
    left: descriptor.bboxPx.left,
    top: descriptor.bboxPx.top,
    width: descriptor.bboxPx.width,
    height: descriptor.bboxPx.height,
    fill: "rgba(0,0,0,0)",
    stroke: descriptor.irObject.kind === "text" ? "#0070f3" : "#00a37b",
    strokeWidth: 1,
    transparentCorners: false,
    lockScalingFlip: true,
  }) as FabricObject;

  controller.metadata = {
    id: descriptor.objectId,
    pageIndex: descriptor.pageIndex,
    baseMatrix: descriptor.matrixPx,
  };

  controller.set({
    a: descriptor.matrixPx[0],
    b: descriptor.matrixPx[1],
    c: descriptor.matrixPx[2],
    d: descriptor.matrixPx[3],
    left: descriptor.matrixPx[4],
    top: descriptor.matrixPx[5],
  });

  controller.setControlsVisibility({ mtr: true });
  controller.setCoords();
  canvas.add(controller);
}

function inferPatchKind(id: string, objects: PdfObjectIR[]): "text" | "image" | "path" {
  const target = objects.find((obj) => obj.id === id);
  return target?.kind ?? "path";
}
