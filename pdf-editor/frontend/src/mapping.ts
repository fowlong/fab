import type { PageIR, PageObject } from './types';
import { multiplyMatrix, ptToPxMatrix } from './coords';

export interface FabricDescriptor {
  id: string;
  pageIndex: number;
  bboxPx: { left: number; top: number; width: number; height: number };
  initialMatrix: [number, number, number, number, number, number];
  kind: PageObject['kind'];
}

const PX_SCALE = 72 / 96;

function resolveMatrix(object: PageObject): [number, number, number, number, number, number] {
  switch (object.kind) {
    case 'text':
      return object.Tm;
    case 'image':
    case 'path':
      return object.cm;
    default:
      return [1, 0, 0, 1, 0, 0];
  }
}

export function mapPageObjectsToFabric(page: PageIR): FabricDescriptor[] {
  const pageMatrix = ptToPxMatrix(page.heightPt);
  return page.objects.map((object) => {
    const [leftPt, bottomPt, rightPt, topPt] = object.bbox;
    const widthPt = rightPt - leftPt;
    const heightPt = topPt - bottomPt;
    const baseMatrix = resolveMatrix(object);
    const transform = multiplyMatrix(pageMatrix, baseMatrix);
    return {
      id: object.id,
      pageIndex: page.index,
      bboxPx: {
        left: leftPt / PX_SCALE,
        top: (page.heightPt - topPt) / PX_SCALE,
        width: widthPt / PX_SCALE,
        height: heightPt / PX_SCALE,
      },
      initialMatrix: transform,
      kind: object.kind,
    };
  });
}
