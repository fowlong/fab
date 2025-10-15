export interface PdfRef {
  obj: number;
  gen: number;
}

export type PdfObjectKind = 'text' | 'image' | 'path';

export interface TextGlyph {
  gid: number;
  dx: number;
  dy: number;
}

export interface TextObject {
  id: string;
  kind: 'text';
  pdfRef: PdfRef;
  btSpan: { start: number; end: number; streamObj: number };
  Tm: [number, number, number, number, number, number];
  font: { resName: string; size: number; type: string };
  unicode: string;
  glyphs: TextGlyph[];
  bbox: [number, number, number, number];
}

export interface ImageObject {
  id: string;
  kind: 'image';
  pdfRef: PdfRef;
  xObject: string;
  cm: [number, number, number, number, number, number];
  bbox: [number, number, number, number];
}

export interface PathObject {
  id: string;
  kind: 'path';
  pdfRef: PdfRef;
  bbox: [number, number, number, number];
}

export type PdfObject = TextObject | ImageObject | PathObject;

export interface PageIR {
  index: number;
  widthPt: number;
  heightPt: number;
  objects: PdfObject[];
}

export interface DocumentIR {
  pages: PageIR[];
}

export interface TransformPatch {
  op: 'transform';
  target: { page: number; id: string };
  deltaMatrixPt: [number, number, number, number, number, number];
  kind: PdfObjectKind;
}

export interface EditTextPatch {
  op: 'editText';
  target: { page: number; id: string };
  text: string;
  fontPref?: { preferExisting: boolean; fallbackFamily?: string };
}

export interface SetStylePatch {
  op: 'setStyle';
  target: { page: number; id: string };
  style: Record<string, unknown>;
}

export type PatchOperation = TransformPatch | EditTextPatch | SetStylePatch;
