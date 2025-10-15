import { fabric } from "fabric";
import { fabricDeltaToPdfDelta, type Matrix } from "./coords";
import { mapPageObjectsToFabric } from "./mapping";
import type { DocumentIR, Page, PageObject, PatchOp } from "./types";
import { postPatch } from "./api";

type OverlayContext = {
  docId: string;
  ir: DocumentIR;
};

const overlayStore = new Map<number, fabric.Canvas>();

export function initFabricOverlay(container: HTMLElement, ctx: OverlayContext) {
  ctx.ir.pages.forEach((page) => {
    const canvasEl = container.querySelector<HTMLCanvasElement>(`#fabric-p${page.index}`);
    if (!canvasEl) {
      return;
    }
    const canvas = new fabric.Canvas(canvasEl, {
      selection: false,
      perPixelTargetFind: true
    });
    overlayStore.set(page.index, canvas);
    attachObjects(canvas, page, ctx);
  });
}

function attachObjects(canvas: fabric.Canvas, page: Page, ctx: OverlayContext) {
  const descriptors = mapPageObjectsToFabric(page);
  descriptors.forEach((desc) => {
    const rect = new fabric.Rect({
      left: desc.bboxPx.left,
      top: desc.bboxPx.top,
      width: desc.bboxPx.width,
      height: desc.bboxPx.height,
      fill: "rgba(0,0,0,0)",
      stroke: "#00aaff",
      strokeWidth: 1,
      selectable: true,
      hasControls: true
    });
    (rect as any).__irObject = desc.object;
    canvas.add(rect);
    (rect as any).__initialMatrix = objectMatrix(rect);
  });

  canvas.on("object:modified", async (ev) => {
    const obj = ev.target as fabric.Object & {
      __irObject?: PageObject;
      __initialMatrix?: Matrix;
    };
    if (!obj || !obj.__irObject || !obj.__initialMatrix) {
      return;
    }
    const { ops, nextMatrix } = buildTransformPatch(obj, page, ctx);
    if (!ops.length) {
      return;
    }
    try {
      await postPatch(ctx.docId, ops);
      obj.__initialMatrix = nextMatrix;
    } catch (err) {
      console.error("Patch failed", err);
    }
  });
}

function buildTransformPatch(
  obj: fabric.Object & { __irObject: PageObject; __initialMatrix: Matrix },
  page: Page,
  ctx: OverlayContext
): { ops: PatchOp[]; nextMatrix: Matrix } {
  const fabricMatrix = objectMatrix(obj);
  const delta = fabricDeltaToPdfDelta(obj.__initialMatrix, fabricMatrix, page.heightPt);
  const kind = obj.__irObject.kind;
  return {
    ops: [
      {
        op: "transform",
        target: { page: page.index, id: obj.__irObject.id },
        deltaMatrixPt: delta,
        kind
      }
    ],
    nextMatrix: fabricMatrix
  };
}

function objectMatrix(obj: fabric.Object): Matrix {
  const { a, b, c, d, e, f } = (obj.calcTransformMatrix() as any) as {
    a: number;
    b: number;
    c: number;
    d: number;
    e: number;
    f: number;
  };
  return [a, b, c, d, e, f];
}
