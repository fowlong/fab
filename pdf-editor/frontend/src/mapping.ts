import type { Matrix } from './coords';
import type { PageObject } from './types';

export interface FabricMeta {
  id: string;
  pageIndex: number;
  baseMatrix: Matrix;
  object: PageObject;
}

export function createFabricMeta(pageIndex: number, object: PageObject, matrix: Matrix): FabricMeta {
  return {
    id: object.id,
    pageIndex,
    baseMatrix: matrix,
    object,
  };
}

