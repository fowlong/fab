import type { IRObject } from "./types";
import { ptToPxMatrix } from "./coords";

export interface FabricMapping {
  left: number;
  top: number;
  width: number;
  height: number;
  matrix: number[];
}

export function mapObjectToFabric(
  obj: IRObject,
  pageHeightPt: number,
): FabricMapping {
  const [x0, y0, x1, y1] = obj.bbox;
  const width = x1 - x0;
  const height = y1 - y0;
  const matrix = ptToPxMatrix(pageHeightPt);
  return {
    left: x0,
    top: y0,
    width,
    height,
    matrix,
  };
}
