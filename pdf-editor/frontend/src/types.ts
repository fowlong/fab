export interface DocumentIR {
  docId: string;
  sourcePdf?: string;
  pages: PageIR[];
}

export interface PageIR {
  index: number;
  widthPt: number;
  heightPt: number;
  objects: DocumentObjectIR[];
}

export type DocumentObjectIR = TextObjectIR | ImageObjectIR | PathObjectIR;

export interface BaseObjectIR {
  id: string;
  pageIndex: number;
  kind: 'text' | 'image' | 'path';
  bbox: [number, number, number, number];
}

export interface TextObjectIR extends BaseObjectIR {
  kind: 'text';
  Tm: number[];
  font: {
    resName: string;
    size: number;
    type: string;
  };
  unicode: string;
}

export interface ImageObjectIR extends BaseObjectIR {
  kind: 'image';
  cm: number[];
  xObject: string;
}

export interface PathObjectIR extends BaseObjectIR {
  kind: 'path';
  cm: number[];
}

export type PatchOperation = TransformPatchOperation | EditTextPatchOperation | SetStylePatchOperation;

export interface PatchTarget {
  page: number;
  id: string;
}

export interface TransformPatchOperation {
  op: 'transform';
  kind: DocumentObjectIR['kind'];
  target: PatchTarget;
  deltaMatrixPt: number[];
}

export interface EditTextPatchOperation {
  op: 'editText';
  target: PatchTarget;
  text: string;
  fontPref?: {
    preferExisting: boolean;
    fallbackFamily?: string;
  };
}

export interface SetStylePatchOperation {
  op: 'setStyle';
  target: PatchTarget;
  style: {
    opacityFill?: number;
    opacityStroke?: number;
    fillColor?: string;
    strokeColor?: string;
  };
}
