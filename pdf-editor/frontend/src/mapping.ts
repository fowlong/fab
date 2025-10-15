import { fabric } from "fabric";
import type { PageIR, PageObject } from "./types";
import { ptToPxMatrix } from "./coords";

type Matrix = [number, number, number, number, number, number];

function attachOriginalTransform(object: fabric.Object, matrix: Matrix) {
  (object as unknown as { __originalTransform?: Matrix }).__originalTransform = matrix;
}

export function mapObjectToFabric(page: PageIR, object: PageObject) {
  const matrix = ptToPxMatrix(page.heightPt);
  const width = object.bboxPx[2] - object.bboxPx[0];
  const height = object.bboxPx[3] - object.bboxPx[1];

  const base = new fabric.Rect({
    width,
    height,
    left: object.bboxPx[0],
    top: object.bboxPx[1],
    fill: "transparent",
    strokeWidth: 1,
    stroke: object.kind === "text" ? "#0a84ff" : "#ff9f0a"
  });

  attachOriginalTransform(base, matrix);
  base.setControlsVisibility({
    mtr: true
  });

  base.lockScalingFlip = true;
  return base;
}
