import { fabric } from "fabric";
import type { DocumentIR, PageObject, PatchOperation } from "./types";
import { fabricDeltaToPdfDelta, type Matrix } from "./coords";
import { mapPageObjects } from "./mapping";

type PatchCallback = (ops: PatchOperation[]) => Promise<void> | void;

export interface FabricObjectMeta {
  id: string;
  kind: PageObject["kind"];
  pageIndex: number;
  baseMatrix: Matrix;
}

interface FabricController {
  object: fabric.Object;
  meta: FabricObjectMeta;
  initialMatrix: Matrix;
}

export class FabricOverlay {
  private host: HTMLElement;
  private canvases: HTMLCanvasElement[];
  private ir: DocumentIR;
  private onPatch: PatchCallback;
  private controllers: FabricController[] = [];
  private fabricCanvases: fabric.Canvas[] = [];

  constructor(host: HTMLElement, canvases: HTMLCanvasElement[], ir: DocumentIR, onPatch: PatchCallback) {
    this.host = host;
    this.canvases = canvases;
    this.ir = ir;
    this.onPatch = onPatch;
  }

  mount() {
    this.unmount();

    this.fabricCanvases = this.canvases.map((canvas, index) => {
      const fabricCanvas = new fabric.Canvas(canvas, {
        selection: false,
        skipTargetFind: false,
      });
      fabricCanvas.upperCanvasEl.classList.add("fabric-overlay");
      fabricCanvas.on("object:modified", (event) => {
        const target = event.target as fabric.Object | undefined;
        if (!target) return;
        const controller = this.controllers.find((ctrl) => ctrl.object === target);
        if (!controller) return;
        void this.handleModified(controller);
      });
      return fabricCanvas;
    });

    this.controllers = this.ir.pages.flatMap((page) => {
      const mapped = mapPageObjects(page);
      const fabricCanvas = this.fabricCanvases[page.index];
      if (!fabricCanvas) return [];

      return mapped.map((mappedObj) => {
        const rect = new fabric.Rect({
          left: mappedObj.bboxPx[0],
          top: mappedObj.bboxPx[1],
          width: mappedObj.bboxPx[2],
          height: mappedObj.bboxPx[3],
          fill: "rgba(0,0,0,0)",
          stroke: "#3b82f6",
          strokeDashArray: [6, 4],
          selectable: true,
        });
        rect.set("meta", mappedObj.meta);
        fabricCanvas.add(rect);
        return {
          object: rect,
          meta: mappedObj.meta,
          initialMatrix: mappedObj.initialMatrix,
        } satisfies FabricController;
      });
    });
  }

  unmount() {
    for (const canvas of this.fabricCanvases) {
      canvas.dispose();
    }
    this.fabricCanvases = [];
    this.controllers = [];
  }

  dispose() {
    this.unmount();
  }

  async applyPatch(ops: PatchOperation[]) {
    await this.onPatch(ops);
  }

  private async handleModified(controller: FabricController) {
    const fabricCanvas = this.fabricCanvases[controller.meta.pageIndex];
    if (!fabricCanvas) return;
    const matrix = controller.object.calcTransformMatrix() as Matrix;
    const delta = fabricDeltaToPdfDelta(controller.initialMatrix, matrix, this.ir.pages[controller.meta.pageIndex].heightPt);

    await this.onPatch([
      {
        op: "transform",
        target: { page: controller.meta.pageIndex, id: controller.meta.id },
        deltaMatrixPt: delta,
        kind: controller.meta.kind,
      },
    ]);
  }
}
