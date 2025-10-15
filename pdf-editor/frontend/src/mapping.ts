import type { IrObject, Page } from "./types";
import { POINTS_PER_PIXEL, Matrix } from "./coords";

export type FabricDescriptor = {
  id: string;
  pageIndex: number;
  bboxPx: { left: number; top: number; width: number; height: number };
  initialMatrix: Matrix;
  kind: IrObject["kind"];
  source: IrObject;
};

export function pagePointSizeToPixels(valuePt: number): number {
  return valuePt / POINTS_PER_PIXEL;
}

export function bboxPtToPx(
  bbox: [number, number, number, number],
  pageHeightPt: number
): { left: number; top: number; width: number; height: number } {
  const [x0, y0, x1, y1] = bbox;
  const width = pagePointSizeToPixels(x1 - x0);
  const height = pagePointSizeToPixels(y1 - y0);
  const left = pagePointSizeToPixels(x0);
  const top = pagePointSizeToPixels(pageHeightPt - y1);
  return { left, top, width, height };
}

export function buildFabricDescriptors(page: Page): FabricDescriptor[] {
  return page.objects.map((obj) => {
    const bboxPx = bboxPtToPx(obj.bbox, page.heightPt);
    let matrix: Matrix;
    if (obj.kind === "text") {
      matrix = obj.Tm;
    } else {
      matrix = obj.cm ?? obj.Tm;
    }
    return {
      id: obj.id,
      pageIndex: page.index,
      bboxPx,
      initialMatrix: matrix,
      kind: obj.kind,
      source: obj,
    };
  });
}
