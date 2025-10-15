export type PdfMatrix = [number, number, number, number, number, number];

export interface PdfRef {
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
  pdfRef: PdfRef;
  btSpan: {
    start: number;
    end: number;
    streamObj: number;
  };
  Tm: PdfMatrix;
  font: {
    resName: string;
    size: number;
    type: string;
  };
  unicode: string;
  glyphs: TextGlyph[];
  bbox: [number, number, number, number];
}

export interface ImageObject {
  id: string;
  kind: 'image';
  pdfRef: PdfRef;
  xObject: string;
  cm: PdfMatrix;
  bbox: [number, number, number, number];
}

export interface PathObject {
  id: string;
  kind: 'path';
  pdfRef: PdfRef;
  cm: PdfMatrix;
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
  deltaMatrixPt: PdfMatrix;
  kind: 'text' | 'image' | 'path';
}

export interface EditTextPatch {
  op: 'editText';
  target: { page: number; id: string };
  text: string;
  fontPref?: {
    preferExisting?: boolean;
    fallbackFamily?: string;
  };
}

export interface SetStylePatch {
  op: 'setStyle';
  target: { page: number; id: string };
  style: {
    fillColor?: string;
    strokeColor?: string;
    opacityFill?: number;
    opacityStroke?: number;
  };
}

export type PatchOperation = TransformPatch | EditTextPatch | SetStylePatch;

export interface PatchResponse {
  ok: boolean;
  updatedPdf?: string;
  remap?: Record<string, { pdfRef: PdfRef }>;
  error?: string;
}
