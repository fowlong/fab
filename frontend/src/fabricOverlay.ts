import { fabric } from "fabric";
import type { PageIR, PageObjectIR, PatchOperation, PatchOperationTransform, PatchOperationEditText } from "./types";
import { mapIrObjectToFabric } from "./mapping";
import { fabricDeltaToPdfDelta, toFabricMatrix } from "./coords";

export interface FabricOverlayCallbacks {
  onTransform?: (ops: PatchOperation[]) => void;
  onEditText?: (ops: PatchOperation[]) => void;
}

export interface FabricOverlayHandle {
  canvas: fabric.Canvas;
  element: HTMLCanvasElement;
}

export function initFabricOverlay(
  baseCanvas: HTMLCanvasElement,
  page: PageIR,
  viewport: any,
  callbacks: FabricOverlayCallbacks,
) : FabricOverlayHandle {
  const overlayCanvas = document.createElement("canvas");
  overlayCanvas.width = baseCanvas.width;
  overlayCanvas.height = baseCanvas.height;
  overlayCanvas.id = `fabric-p${page.index}`;
  overlayCanvas.className = "fabric-layer";

  const fabricCanvas = new fabric.Canvas(overlayCanvas, {
    selection: false,
    preserveObjectStacking: true,
  });

  page.objects.forEach((obj) => {
    const fabricObj = mapIrObjectToFabric(obj, viewport);
    if (!fabricObj) {
      return;
    }

    (fabricObj as any).irObject = obj;
    (fabricObj as any).initialMatrix = toFabricMatrix(fabricObj.calcTransformMatrix());

    fabricObj.on("modified", () => {
      const initial = (fabricObj as any).initialMatrix as number[];
      const current = toFabricMatrix(fabricObj.calcTransformMatrix());
      const deltaMatrixPt = fabricDeltaToPdfDelta(initial, current, page.heightPt);
      const op: PatchOperationTransform = {
        op: "transform",
        target: { page: page.index, id: obj.id },
        deltaMatrixPt,
        kind: obj.kind,
      };
      callbacks.onTransform?.([op]);
      (fabricObj as any).initialMatrix = current;
    });

    fabricCanvas.add(fabricObj);
  });

  fabricCanvas.on("mouse:dblclick", (event) => {
    const target = event.target as fabric.Object & { irObject?: PageObjectIR } | undefined;
    if (!target || !target.irObject || target.irObject.kind !== "text") {
      return;
    }

    const currentValue = target.irObject.unicode ?? "";
    const nextValue = window.prompt("Edit text", currentValue);
    if (!nextValue || nextValue === currentValue) {
      return;
    }

    const editOp: PatchOperationEditText = {
      op: "editText",
      target: { page: page.index, id: target.irObject.id },
      text: nextValue,
      fontPref: { preferExisting: true },
      kind: "text",
    };

    callbacks.onEditText?.([editOp]);
  });

  return { canvas: fabricCanvas, element: overlayCanvas };
}
