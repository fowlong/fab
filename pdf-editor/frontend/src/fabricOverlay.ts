import { fabric } from "fabric";
import { DocumentIR, PageIR } from "./types";
import { Matrix, pxToPtMatrix } from "./coords";

interface OverlayHandle {
  canvases: Record<number, fabric.Canvas>;
}

export function initFabricOverlay(ir: DocumentIR, preview: { canvases: HTMLCanvasElement[] }): OverlayHandle {
  const canvases: Record<number, fabric.Canvas> = {};

  ir.pages.forEach((page: PageIR, index) => {
    const underlay = preview.canvases[index];
    if (!underlay) {
      return;
    }

    const overlayCanvas = document.createElement("canvas");
    overlayCanvas.id = `fabric-overlay-${index}`;
    overlayCanvas.width = underlay.width;
    overlayCanvas.height = underlay.height;
    underlay.parentElement?.appendChild(overlayCanvas);

    const fabricCanvas = new fabric.Canvas(overlayCanvas, {
      selection: false
    });

    canvases[index] = fabricCanvas;

    page.objects.forEach((obj) => {
      const rect = new fabric.Rect({
        left: 0,
        top: 0,
        width: Math.abs(obj.bbox[2] - obj.bbox[0]),
        height: Math.abs(obj.bbox[3] - obj.bbox[1]),
        stroke: obj.kind === "text" ? "#1d4ed8" : "#059669",
        strokeWidth: 1,
        fill: "",
        selectable: true,
        hasBorders: true,
        hasControls: false
      });

      const matrixPt: Matrix = [1, 0, 0, 1, obj.bbox[0], obj.kind === "text" ? obj.bbox[1] : obj.bbox[1]];
      const pxMatrix = pxToPtMatrix(page.heightPt);
      const { scaleX, scaleY } = fabric.util.qrDecompose({
        a: pxMatrix[0],
        b: pxMatrix[1],
        c: pxMatrix[2],
        d: pxMatrix[3]
      });

      rect.set({
        left: obj.bbox[0] * scaleX,
        top: (page.heightPt - obj.bbox[1]) * scaleY
      });

      fabricCanvas.add(rect);
    });
  });

  return { canvases };
}
