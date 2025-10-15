export interface PdfRef {
  obj: number;
  gen: number;
}

export type ObjectKind = 'text' | 'image' | 'path';

export interface TextObject {
  id: string;
  kind: 'text';
  pdfRef: PdfRef;
  btSpan: {
    start: number;
    end: number;
    streamObj: number;
  };
  Tm: [number, number, number, number, number, number];
  font: {
    resName: string;
    size: number;
    type: string;
  };
  unicode: string;
  glyphs: Array<{ gid: number; dx: number; dy: number }>;
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
  cm: [number, number, number, number, number, number];
  bbox: [number, number, number, number];
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

export interface TransformPatch {
  op: 'transform';
  target: { page: number; id: string };
  deltaMatrixPt: [number, number, number, number, number, number];
  kind: ObjectKind;
}

export interface EditTextPatch {
  op: 'editText';
  target: { page: number; id: string };
  text: string;
  fontPref: {
    preferExisting: boolean;
    fallbackFamily: string;
  };
}

export interface SetStylePatch {
  op: 'setStyle';
  target: { page: number; id: string };
  style: {
    opacityFill?: number;
    opacityStroke?: number;
    fillColor?: string;
    strokeColor?: string;
  };
}

export type PatchOp = TransformPatch | EditTextPatch | SetStylePatch;

export interface PatchResponse {
  ok: boolean;
  updatedPdf?: string;
  remap?: Record<string, { pdfRef: PdfRef }>;
  message?: string;
}
