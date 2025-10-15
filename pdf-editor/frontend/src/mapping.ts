import type { FabricObjectMeta } from "./fabricOverlay";
import type { PageIR, PageObject } from "./types";
import { multiply, ptToPx } from "./coords";

export interface MappedObject {
  meta: FabricObjectMeta;
  bboxPx: [number, number, number, number];
  initialMatrix: [number, number, number, number, number, number];
}

export function mapPageObjects(page: PageIR): MappedObject[] {
  const toPx = ptToPx(page.heightPt);

  return page.objects.map((obj) => {
    const [x0, y0, x1, y1] = obj.bbox;
    const topLeft = transformPoint(toPx, x0, y1);
    const bottomRight = transformPoint(toPx, x1, y0);

    const width = bottomRight[0] - topLeft[0];
    const height = bottomRight[1] - topLeft[1];

    const initialMatrix = multiply(toPx, getObjectMatrix(obj));
    const meta: FabricObjectMeta = {
      id: obj.id,
      kind: obj.kind,
      pageIndex: page.index,
      baseMatrix: getObjectMatrix(obj),
    };

    return {
      meta,
      bboxPx: [topLeft[0], topLeft[1], width, height],
      initialMatrix,
    };
  });
}

function getObjectMatrix(obj: PageObject): [number, number, number, number, number, number] {
  switch (obj.kind) {
    case "text":
      return obj.Tm;
    case "image":
    case "path":
      return obj.cm;
    default:
      return [1, 0, 0, 1, 0, 0];
  }
}

function transformPoint(m: [number, number, number, number, number, number], x: number, y: number): [number, number] {
  const [a, b, c, d, e, f] = m;
  return [a * x + c * y + e, b * x + d * y + f];
}
