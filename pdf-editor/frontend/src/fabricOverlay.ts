import { fabric } from "fabric";
import type { ApiClient } from "./api";
import type { EditorMapping } from "./mapping";
import { fabricDeltaToPdfDelta } from "./coords";
import type { PageIR, TransformPatch } from "./types";

interface OverlayOptions {
  canvasElement: HTMLCanvasElement;
  page: PageIR;
  docId: string;
  api: ApiClient;
  mapping: EditorMapping;
}

type FabricOverlayObject = fabric.Object & {
  irId?: string;
  baseTransform?: number[];
};

const toAffine = (matrix: number[]): number[] => {
  if (matrix.length >= 8) {
    return [matrix[0], matrix[1], matrix[3], matrix[4], matrix[6], matrix[7]];
  }
  return matrix.slice(0, 6);
};

export async function initFabricOverlay({
  canvasElement,
  page,
  docId,
  api,
  mapping
}: OverlayOptions) {
  const canvas = new fabric.Canvas(canvasElement, {
    selection: false,
    preserveObjectStacking: true
  });

  page.objects.forEach((obj) => {
    const controller = mapping.createController(obj, page) as FabricOverlayObject;
    canvas.add(controller);
    controller.baseTransform = toAffine(controller.calcTransformMatrix());
  });

  canvas.on("object:modified", async (evt) => {
    const target = evt.target as FabricOverlayObject;
    if (!target || !target.irId || !target.baseTransform) return;

    const base = target.baseTransform;
    const current = toAffine(target.calcTransformMatrix());
    const delta = fabricDeltaToPdfDelta(base, current, page.heightPt);

    const patch: TransformPatch = {
      op: "transform",
      kind: (target.data as { kind?: "text" | "image" | "path" } | undefined)?.kind ?? "text",
      target: { page: page.index, id: target.irId },
      deltaMatrixPt: delta
    };

    await api.sendPatch(docId, [patch]);
    target.baseTransform = current;
  });
}
