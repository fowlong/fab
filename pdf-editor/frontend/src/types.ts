export interface DocumentIR {
  docId?: string;
  meta?: {
    pdfData?: string; // data URL for previewing
    pageCount?: number;
  };
  pages: PageIR[];
}

export interface PageIR {
  index: number;
  widthPt: number;
  heightPt: number;
  objects: PdfObjectIR[];
}

export type PdfObjectIR = TextObjectIR | ImageObjectIR | PathObjectIR;

export interface BaseObjectIR {
  id: string;
  kind: "text" | "image" | "path";
  bbox: [number, number, number, number];
}

export interface TextObjectIR extends BaseObjectIR {
  kind: "text";
  pdfRef: PdfRef;
  btSpan: { start: number; end: number; streamObj: number };
  Tm: Matrix;
  font: {
    resName: string;
    size: number;
    type: string;
  };
  unicode: string;
  glyphs: Array<{ gid: number; dx: number; dy: number }>;
}

export interface ImageObjectIR extends BaseObjectIR {
  kind: "image";
  pdfRef: PdfRef;
  xObject: string;
  cm: Matrix;
}

export interface PathObjectIR extends BaseObjectIR {
  kind: "path";
  pdfRef: PdfRef;
  cm: Matrix;
}

export interface PdfRef {
  obj: number;
  gen: number;
}

export type Matrix = [number, number, number, number, number, number];

export interface PatchTransform {
  op: "transform";
  target: PatchTarget;
  deltaMatrixPt: Matrix;
  kind: "text" | "image" | "path";
}

export interface PatchEditText {
  op: "editText";
  target: PatchTarget;
  text: string;
  fontPref?: {
    preferExisting?: boolean;
    fallbackFamily?: string;
  };
}

export interface PatchSetStyle {
  op: "setStyle";
  target: PatchTarget;
  style: {
    opacityFill?: number;
    opacityStroke?: number;
    fillColor?: string;
    strokeColor?: string;
  };
}

export type PatchOp = PatchTransform | PatchEditText | PatchSetStyle;

export interface PatchTarget {
  page: number;
  id: string;
}

export interface PatchResponse {
  ok: boolean;
  updatedPdf?: string;
  remap?: Record<string, { pdfRef: PdfRef }>;
}
