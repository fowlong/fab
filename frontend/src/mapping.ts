import { fabric } from "fabric";
import type { IrObject } from "./types";
import { PX_TO_PT_SCALE, pxToPtMatrix } from "./coords";

interface EditorLayout {
  uploadInput: HTMLInputElement;
  previewContainer: HTMLElement;
  overlayContainer: HTMLElement;
}

export function createEditorLayout(root: HTMLElement): EditorLayout {
  root.innerHTML = `
    <div class="app-shell">
      <aside class="sidebar">
        <label class="upload">
          <span>Open PDF</span>
          <input type="file" accept="application/pdf" />
        </label>
        <button id="download-btn">Download current PDF</button>
      </aside>
      <main class="editor">
        <div class="page-stack">
          <div class="preview"></div>
          <div class="overlay"></div>
        </div>
      </main>
    </div>
  `;

  const uploadInput = root.querySelector("input[type=file]") as HTMLInputElement;
  const previewContainer = root.querySelector(".preview") as HTMLElement;
  const overlayContainer = root.querySelector(".overlay") as HTMLElement;

  return { uploadInput, previewContainer, overlayContainer };
}

export function createFabricObjectFromIr(ir: IrObject, pageHeightPt: number): fabric.Rect {
  const bbox = ir.bbox ?? [0, 0, 10, 10];
  const [x0, y0, x1, y1] = bbox;
  const width = (x1 - x0) / PX_TO_PT_SCALE;
  const height = (y1 - y0) / PX_TO_PT_SCALE;
  const pxToPt = pxToPtMatrix(pageHeightPt);
  const inv = fabric.util.invertTransform(pxToPt as any);
  const topLeftPx = fabric.util.transformPoint(new fabric.Point(x0, y1), inv);

  const rect = new fabric.Rect({
    left: topLeftPx.x,
    top: topLeftPx.y,
    width,
    height,
    fill: "rgba(0,0,0,0)",
    stroke: "#2b6cb0",
    strokeWidth: 1,
    originX: "left",
    originY: "top",
    selectable: true,
    hasRotatingPoint: true
  });

  return rect;
}
