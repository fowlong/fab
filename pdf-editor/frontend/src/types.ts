export type Matrix = [number, number, number, number, number, number];

export interface PdfRef {
  obj: number;
  gen: number;
}

export interface BtSpan {
  start: number;
  end: number;
  streamObj: number;
}

export interface GlyphRun {
  gid: number;
  dx: number;
  dy: number;
}

export interface FontDescriptor {
  resName: string;
  size: number;
  type: string;
}

export interface TextObject {
  kind: 'text';
  id: string;
  pdfRef: PdfRef;
  btSpan: BtSpan;
  Tm: Matrix;
  font: FontDescriptor;
  unicode: string;
  glyphs: GlyphRun[];
  bbox: [number, number, number, number];
}

export interface ImageObject {
  kind: 'image';
  id: string;
  pdfRef: PdfRef;
  xObject: string;
  cm: Matrix;
  bbox: [number, number, number, number];
}

export interface PathObject {
  kind: 'path';
  id: string;
  pdfRef: PdfRef;
  cm: Matrix;
  bbox: [number, number, number, number];
}

export type PageObject = TextObject | ImageObject | PathObject;

export interface PageIR {
  index: number;
  widthPt: number;
  heightPt: number;
  objects: PageObject[];
}

export interface DocumentIr {
  pages: PageIR[];
}

export interface PatchTarget {
  page: number;
  id: string;
}

export interface TransformPatch {
  op: 'transform';
  target: PatchTarget;
  deltaMatrixPt: Matrix;
  kind?: 'text' | 'image' | 'path';
}

export interface EditTextPatch {
  op: 'editText';
  target: PatchTarget;
  text: string;
  fontPref?: {
    preferExisting?: boolean;
    fallbackFamily?: string;
  };
}

export interface StylePatch {
  op: 'setStyle';
  target: PatchTarget;
  style: {
    colorFill?: [number, number, number];
    colorStroke?: [number, number, number];
    opacityFill?: number;
    opacityStroke?: number;
  };
}

export type PatchOp = TransformPatch | EditTextPatch | StylePatch;

export interface PatchResponse {
  ok: boolean;
  updatedPdf?: string;
  remap: Record<string, { pdfRef?: PdfRef }>;
}

export interface OpenResponse {
  docId: string;
}
