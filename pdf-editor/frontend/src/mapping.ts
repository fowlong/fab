import type { PdfObject } from './types';

export interface FabricMapping {
  id: string;
  fabricId: string;
}

export class MappingStore {
  private forward = new Map<string, string>();
  private reverse = new Map<string, string>();

  set(map: FabricMapping) {
    this.forward.set(map.id, map.fabricId);
    this.reverse.set(map.fabricId, map.id);
  }

  byFabricId(id: string) {
    return this.reverse.get(id);
  }

  byObjectId(id: string) {
    return this.forward.get(id);
  }
}

export function bboxToFabricRect(obj: PdfObject, pageHeightPt: number) {
  const [x0, y0, x1, y1] = obj.bbox;
  return {
    left: x0,
    top: pageHeightPt - y1,
    width: x1 - x0,
    height: y1 - y0,
  };
}
