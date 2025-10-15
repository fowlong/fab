export interface DocumentMeta {
  fileName: string;
  pageCount: number;
  originalPdfBytes: Uint8Array;
  originalPdf?: string;
}

export interface PdfRef {
  obj: number;
  gen: number;
}

export type PageObjectIR = TextObjectIR | ImageObjectIR | PathObjectIR;

export interface PageIR {
  index: number;
  widthPt: number;
  heightPt: number;
  objects: PageObjectIR[];
}

export interface DocumentIR {
  documentMeta: DocumentMeta;
  pages: PageIR[];
}

export interface TextObjectIR {
  id: string;
  kind: "text";
  pdfRef?: PdfRef;
  unicode?: string;
  Tm?: number[];
  font?: {
    resName: string;
    size: number;
    type: string;
  };
  glyphs?: Array<{ gid: number; dx: number; dy: number }>;
  bbox: [number, number, number, number];
}

export interface ImageObjectIR {
  id: string;
  kind: "image";
  pdfRef?: PdfRef;
  xObject?: string;
  cm?: number[];
  bbox: [number, number, number, number];
}

export interface PathObjectIR {
  id: string;
  kind: "path";
  pdfRef?: PdfRef;
  cm?: number[];
  bbox: [number, number, number, number];
}

export interface PatchTarget {
  page: number;
  id: string;
}

export interface PatchOperationTransform {
  op: "transform";
  kind: PageObjectIR["kind"];
  target: PatchTarget;
  deltaMatrixPt: number[];
}

export interface PatchOperationEditText {
  op: "editText";
  kind: "text";
  target: PatchTarget;
  text: string;
  fontPref?: {
    preferExisting?: boolean;
    fallbackFamily?: string;
  };
}

export interface PatchOperationSetStyle {
  op: "setStyle";
  target: PatchTarget;
  style: {
    colourFill?: [number, number, number];
    opacityFill?: number;
  };
}

export type PatchOperation =
  | PatchOperationTransform
  | PatchOperationEditText
  | PatchOperationSetStyle;

export interface PatchResponse {
  ok: boolean;
  updatedPdf?: string;
  remap?: Record<string, unknown>;
}
