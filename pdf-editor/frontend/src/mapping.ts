import type { Matrix } from "./coords";
import { multiply, ptToPxMatrix } from "./coords";
import type { PageObject, Page } from "./types";

export type FabricDescriptor = {
  id: string;
  pageIndex: number;
  bboxPx: { left: number; top: number; width: number; height: number };
  transform: Matrix;
  object: PageObject;
};

export function mapPageObjectsToFabric(page: Page): FabricDescriptor[] {
  const ptToPx = ptToPxMatrix(page.heightPt);
  return page.objects.map((object) => {
    const [x1, y1, x2, y2] = object.bbox;
    const widthPt = x2 - x1;
    const heightPt = y2 - y1;
    const bboxMatrix: Matrix = [1, 0, 0, 1, x1, y1];
    const transform = multiply(ptToPx, bboxMatrix);
    const leftTop = applyMatrix(transform, 0, heightPt);
    const rightBottom = applyMatrix(transform, widthPt, 0);
    return {
      id: object.id,
      pageIndex: page.index,
      bboxPx: {
        left: Math.min(leftTop.x, rightBottom.x),
        top: Math.min(leftTop.y, rightBottom.y),
        width: Math.abs(rightBottom.x - leftTop.x),
        height: Math.abs(rightBottom.y - leftTop.y)
      },
      transform,
      object
    };
  });
}

export function applyMatrix(matrix: Matrix, x: number, y: number): { x: number; y: number } {
  const [a, b, c, d, e, f] = matrix;
  return {
    x: a * x + c * y + e,
    y: b * x + d * y + f
  };
}
