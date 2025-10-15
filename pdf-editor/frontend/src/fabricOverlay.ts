import { fabric } from "fabric";
import type { Matrix } from "./coords";
import { fabricDeltaToPdfDelta } from "./coords";
import type { PageIR, PageObject, PatchOp } from "./types";
import { objectToFabric } from "./mapping";
import { postPatch } from "./api";

export interface OverlayContext {
  docId: string;
  onPatch?: (ops: PatchOp[]) => void;
}

interface FabricMeta {
  id: string;
  pageIndex: number;
  baseMatrix: Matrix;
  pageHeightPt: number;
}

const metaMap = new WeakMap<fabric.Object, FabricMeta>();

export async function initOverlay(
  container: HTMLElement,
  page: PageIR,
  ctx: OverlayContext,
) {
  const fabricCanvas = new fabric.Canvas(document.createElement("canvas"), {
    selection: false,
    perPixelTargetFind: true,
  });
  fabricCanvas.setWidth(container.clientWidth);
  fabricCanvas.setHeight(container.clientHeight);
  container.appendChild(fabricCanvas.getElement() as HTMLCanvasElement);

  page.objects.forEach((obj) => addControllerForObject(fabricCanvas, obj, page, ctx));

  fabricCanvas.on("object:modified", (evt) => {
    const target = evt.target;
    if (!target) return;
    const meta = metaMap.get(target);
    if (!meta) return;

    const patch: PatchOp = {
      op: "transform",
      target: { page: meta.pageIndex, id: meta.id },
      deltaMatrixPt: fabricDeltaToPdfDelta(
        meta.baseMatrix,
        target.calcTransformMatrix() as Matrix,
        meta.pageHeightPt,
      ),
      kind: inferKind(target),
    } as PatchOp;

    ctx.onPatch?.([patch]);
    postPatch(ctx.docId, [patch]).catch((err) => {
      console.error("Failed to post transform patch", err);
    });
  });
}

function inferKind(obj: fabric.Object): PageObject["kind"] {
  const meta = metaMap.get(obj);
  if (!meta) {
    return "path";
  }
  const id = meta.id;
  if (id.startsWith("t:")) return "text";
  if (id.startsWith("img:")) return "image";
  return "path";
}

function addControllerForObject(
  canvas: fabric.Canvas,
  obj: PageObject,
  page: PageIR,
  ctx: OverlayContext,
) {
  const descriptor = objectToFabric(obj, page.heightPt);
  const rect = new fabric.Rect({
    left: descriptor.bboxPx.left,
    top: descriptor.bboxPx.top,
    width: descriptor.bboxPx.width,
    height: descriptor.bboxPx.height,
    fill: "rgba(0,0,0,0)",
    stroke: "#4f83ff",
    strokeWidth: 1,
    transparentCorners: false,
    cornerColor: "#4f83ff",
  });
  rect.set({
    transformMatrix: descriptor.matrixPx,
    lockScalingFlip: true,
  });

  metaMap.set(rect, {
    id: obj.id,
    pageIndex: page.index,
    baseMatrix: descriptor.matrixPx,
    pageHeightPt: page.heightPt,
  });

  canvas.add(rect);

  rect.on("mousedblclick", () => {
    if (obj.kind !== "text") return;
    ctx.onPatch?.([
      {
        op: "editText",
        target: { page: page.index, id: obj.id },
        text: obj.unicode,
      },
    ]);
  });
}
