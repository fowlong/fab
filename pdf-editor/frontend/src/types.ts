export interface PdfReference {
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
  pdfRef: PdfReference;
  btSpan: {
    start: number;
    end: number;
    streamObj: number;
  };
  Tm: Matrix;
  font: {
    resName: string;
    size: number;
    type: string;
  };
  unicode: string;
  glyphs: TextGlyph[];
  bbox: BoundingBox;
}

export interface ImageObject {
  id: string;
  kind: 'image';
  pdfRef: PdfReference;
  xObject: string;
  cm: Matrix;
  bbox: BoundingBox;
}

export interface PathObject {
  id: string;
  kind: 'path';
  pdfRef: PdfReference;
  operations: string[];
  cm: Matrix;
  bbox: BoundingBox;
}

export type PageObject = TextObject | ImageObject | PathObject;

export interface PageIR {
  index: number;
  widthPt: number;
  heightPt: number;
  objects: PageObject[];
}

export interface DocumentIR {
  pages: PageIR[];
}

export type Matrix = [number, number, number, number, number, number];
export type BoundingBox = [number, number, number, number];

export interface TransformPatch {
  op: 'transform';
  target: PatchTarget;
  deltaMatrixPt: Matrix;
  kind: 'text' | 'image' | 'path';
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

export interface SetStylePatch {
  op: 'setStyle';
  target: PatchTarget;
  style: Partial<{
    fillColor: string;
    strokeColor: string;
    opacityFill: number;
    opacityStroke: number;
  }>;
}

export type PatchOperation = TransformPatch | EditTextPatch | SetStylePatch;

export interface PatchTarget {
  page: number;
  id: string;
}

export interface PatchResponse {
  ok: boolean;
  updatedPdf?: string;
  remap?: Record<string, { pdfRef: PdfReference }>;
  error?: string;
}
