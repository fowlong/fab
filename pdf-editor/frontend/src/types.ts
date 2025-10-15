export type Matrix = [number, number, number, number, number, number];

export interface PdfObjectRef {
  obj: number;
  gen: number;
}

export interface TextGlyph {
  gid: number;
  dx: number;
  dy: number;
}

export interface TextObject {
  id: string;
  kind: 'text';
  pdfRef: PdfObjectRef;
  btSpan: { start: number; end: number; streamObj: number };
  Tm: Matrix;
  font: { resName: string; size: number; type: string };
  unicode: string;
  glyphs: TextGlyph[];
  bbox: [number, number, number, number];
}

export interface ImageObject {
  id: string;
  kind: 'image';
  pdfRef: PdfObjectRef;
  xObject: string;
  cm: Matrix;
  bbox: [number, number, number, number];
}

export interface PathObject {
  id: string;
  kind: 'path';
  pdfRef: PdfObjectRef;
  cm: Matrix;
  bbox: [number, number, number, number];
}

export type IrObject = TextObject | ImageObject | PathObject;

export interface PageIr {
  index: number;
  widthPt: number;
  heightPt: number;
  objects: IrObject[];
}

export interface DocumentIr {
  pages: PageIr[];
  sourcePdfUrl?: string;
}

export interface TransformPatchOperation {
  op: 'transform';
  target: { page: number; id: string };
  deltaMatrixPt: Matrix;
  kind: 'text' | 'image' | 'path' | 'generic';
}

export interface EditTextPatchOperation {
  op: 'editText';
  target: { page: number; id: string };
  text: string;
  fontPref?: { preferExisting: boolean; fallbackFamily?: string };
}

export interface SetStylePatchOperation {
  op: 'setStyle';
  target: { page: number; id: string };
  style: Record<string, unknown>;
}

export type PatchOperation =
  | TransformPatchOperation
  | EditTextPatchOperation
  | SetStylePatchOperation;

export interface PatchResponse {
  ok: boolean;
  updatedPdf?: string;
  remap?: Record<string, { pdfRef: PdfObjectRef }>;
}
