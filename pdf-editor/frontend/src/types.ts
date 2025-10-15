export interface DocumentIr {
  pages: PageIr[];
  meta?: Record<string, unknown>;
}

export interface PageIr {
  index: number;
  widthPt: number;
  heightPt: number;
  objects: PageObject[];
}

export type PageObject = TextObject | ImageObject | PathObject;

export interface PdfRef {
  obj: number;
  gen: number;
}

export interface StreamSpan {
  start: number;
  end: number;
  streamObj: number;
}

export interface TextGlyph {
  gid: number;
  dx: number;
  dy: number;
}

export interface TextFont {
  resName: string;
  size: number;
  type?: string;
}

export interface TextObject {
  kind: 'text';
  id: string;
  pdfRef?: PdfRef;
  btSpan?: StreamSpan;
  tm: [number, number, number, number, number, number];
  font?: TextFont;
  unicode: string;
  glyphs?: TextGlyph[];
  bbox: [number, number, number, number];
}

export interface ImageObject {
  kind: 'image';
  id: string;
  pdfRef?: PdfRef;
  xObject?: string;
  cm: [number, number, number, number, number, number];
  bbox: [number, number, number, number];
}

export interface PathObject {
  kind: 'path';
  id: string;
  pdfRef?: PdfRef;
  cm: [number, number, number, number, number, number];
  bbox: [number, number, number, number];
  style?: Record<string, unknown>;
}

export type PatchOp = TransformOp | EditTextOp | SetStyleOp;

export interface PatchTarget {
  page: number;
  id: string;
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
  fontPref?: FontPreference;
}

export interface FontPreference {
  preferExisting?: boolean;
  fallbackFamily?: string;
}

export interface SetStyleOp {
  op: 'setStyle';
  target: PatchTarget;
  style: Record<string, unknown>;
}

export interface PatchResponse {
  ok: boolean;
  updatedPdf?: string;
  remap?: Record<string, { pdfRef: PdfRef }>;
}

export type DocumentId = string;

export interface OpenResponse {
  doc_id: DocumentId;
}

export interface OpenRequest {
  pdf_base64: string;
}

declare const __API_BASE__: string;

export { __API_BASE__ };
