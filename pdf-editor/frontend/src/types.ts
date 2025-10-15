export interface PdfRef {
  obj: number;
  gen: number;
}

export type IrObject =
  | TextObject
  | ImageObject
  | PathObject;

export interface BaseObject {
  id: string;
  kind: 'text' | 'image' | 'path';
  bbox: [number, number, number, number];
  pdfRef?: PdfRef;
}

export interface TextObject extends BaseObject {
  kind: 'text';
  unicode: string;
  Tm?: [number, number, number, number, number, number];
}

export interface ImageObject extends BaseObject {
  kind: 'image';
  cm?: [number, number, number, number, number, number];
}

export interface PathObject extends BaseObject {
  kind: 'path';
  cm?: [number, number, number, number, number, number];
}

export interface IrPage {
  index: number;
  widthPt: number;
  heightPt: number;
  objects: IrObject[];
}

export interface IrDocument {
  pages: IrPage[];
}

export interface TransformOp {
  op: 'transform';
  target: PatchTarget;
  deltaMatrixPt: [number, number, number, number, number, number];
  kind: 'text' | 'image' | 'path';
}

export interface EditTextOp {
  op: 'editText';
  target: PatchTarget;
  text: string;
}

export interface SetStyleOp {
  op: 'setStyle';
  target: PatchTarget;
  style: Record<string, unknown>;
}

export type PatchOperation = TransformOp | EditTextOp | SetStyleOp;

export interface PatchTarget {
  page: number;
  id: string;
}

export interface OpenResponse {
  docId: string;
}

export interface PatchResponse {
  ok: boolean;
  updatedPdf?: string;
  remap?: Record<string, unknown>;
}
