import { fabric } from "fabric";
import { fabricDeltaToPdfDelta } from "./coords";
import type { IrDocument, IrObject, PatchOp } from "./types";
import { bboxToFabricRect } from "./mapping";

interface OverlayEntry {
  canvas: fabric.Canvas;
  pageHeightPt: number;
  objects: Map<string, {
    rect: fabric.Rect;
    baseMatrix: number[];
    ir: IrObject;
  }>;
}

export type PatchHandler = (ops: PatchOp[]) => Promise<void>;

export class OverlayManager {
  private overlays = new Map<number, OverlayEntry>();

  constructor(private readonly patchHandler: PatchHandler) {}

  attachToCanvases(
    fabricCanvases: HTMLCanvasElement[],
    ir: IrDocument
  ) {
    ir.pages.forEach((page) => {
      const overlayCanvas = fabricCanvases[page.index];
      const canvas = new fabric.Canvas(overlayCanvas, {
        selection: false,
        preserveObjectStacking: true
      });
      const entry: OverlayEntry = {
        canvas,
        pageHeightPt: page.heightPt,
        objects: new Map()
      };

      page.objects.forEach((obj) => {
        const rect = bboxToFabricRect(obj, page.heightPt);
        rect.setControlsVisibility({
          mt: false,
          mb: false,
          ml: false,
          mr: false
        });
        canvas.add(rect);
        entry.objects.set(obj.id, {
          rect,
          baseMatrix: fabricMatrixFromObject(rect),
          ir: obj
        });
        rect.on("modified", () => {
          this.onObjectModified(page.index, obj.id);
        });
      });

      this.overlays.set(page.index, entry);
    });
  }

  private async onObjectModified(pageIndex: number, objectId: string) {
    const entry = this.overlays.get(pageIndex);
    if (!entry) return;
    const meta = entry.objects.get(objectId);
    if (!meta) return;
    const rect = meta.rect;
    const fabricMatrix = fabricMatrixFromObject(rect);
    const delta = fabricDeltaToPdfDelta(
      meta.baseMatrix as number[],
      fabricMatrix as number[],
      entry.pageHeightPt
    );
    const op: PatchOp = {
      op: "transform",
      target: { page: pageIndex, id: objectId },
      deltaMatrixPt: delta as any,
      kind: meta.ir.kind
    };
    await this.patchHandler([op]);
    meta.baseMatrix = fabricMatrix;
    rect.setCoords();
  }
}

function fabricMatrixFromObject(object: fabric.Object): number[] {
  const t = object.calcTransformMatrix();
  return [t[0], t[1], t[4], t[5], t[12], t[13]];
}
