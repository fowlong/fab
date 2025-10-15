import { fabric } from "fabric";
import type { Matrix } from "./coords";
import { fabricDeltaToPdfDelta } from "./coords";
import type { FabricDescriptor } from "./mapping";
import type { PatchOperation } from "./types";

export type OverlayCallbacks = {
  onTransform: (ops: PatchOperation[]) => void;
  onTextEdit: (targetId: string, text: string) => void;
};

export class FabricOverlay {
  private canvases = new Map<number, fabric.Canvas>();

  constructor(private readonly callbacks: OverlayCallbacks) {}

  attach(pageIndex: number, canvasElement: HTMLCanvasElement): void {
    const overlay = new fabric.Canvas(canvasElement, {
      selection: true,
      preserveObjectStacking: true,
    });
    overlay.setDimensions({ width: canvasElement.width, height: canvasElement.height });
    overlay.getElement().classList.add("fabric-overlay");
    overlay.on("object:modified", (evt) => {
      const target = evt.target as fabric.Object & { irId?: string; pageHeightPt?: number; initialMatrix?: Matrix };
      if (!target || !target.irId || !target.pageHeightPt || !target.initialMatrix) {
        return;
      }
      const delta = fabricDeltaToPdfDelta(
        target.initialMatrix as Matrix,
        target.calcTransformMatrix() as Matrix,
        target.pageHeightPt as number
      );
      const kind = (target as any).irKind ?? "text";
      const op: PatchOperation = {
        op: "transform",
        target: { page: pageIndex, id: target.irId },
        deltaMatrixPt: delta,
        kind,
      } as PatchOperation;
      this.callbacks.onTransform([op]);
      target.initialMatrix = target.calcTransformMatrix() as Matrix;
    });
    this.canvases.set(pageIndex, overlay);
  }

  clear(pageIndex: number): void {
    this.canvases.get(pageIndex)?.clear();
  }

  dispose(): void {
    this.canvases.forEach((c) => c.dispose());
    this.canvases.clear();
  }

  renderDescriptors(pageIndex: number, descriptors: FabricDescriptor[], pageHeightPt: number): void {
    const canvas = this.canvases.get(pageIndex);
    if (!canvas) {
      throw new Error(`No fabric canvas for page ${pageIndex}`);
    }
    canvas.clear();
    descriptors.forEach((descriptor) => {
      const rect = new fabric.Rect({
        left: descriptor.bboxPx.left,
        top: descriptor.bboxPx.top,
        width: descriptor.bboxPx.width,
        height: descriptor.bboxPx.height,
        fill: "rgba(0,0,0,0)",
        stroke: descriptor.kind === "text" ? "#4b9eff" : "#ffb54b",
        strokeWidth: 1,
        selectable: true,
        evented: true,
        hasBorders: true,
        hasControls: true,
      }) as fabric.Rect & {
        irId?: string;
        pageHeightPt?: number;
        initialMatrix?: Matrix;
        irKind?: string;
      };
      rect.irId = descriptor.id;
      rect.irKind = descriptor.kind;
      rect.pageHeightPt = pageHeightPt;
      canvas.add(rect);
      rect.initialMatrix = rect.calcTransformMatrix() as Matrix;
    });
    canvas.renderAll();
  }
}
