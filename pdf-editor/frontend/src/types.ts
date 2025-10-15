export interface PdfRef {
  obj: number;
  gen: number;
}

export type PdfObjectId = string;

export interface Glyph {
  gid: number;
  dx: number;
  dy: number;
}

export interface TextObject {
  id: PdfObjectId;
  kind: "text";
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
  glyphs: Glyph[];
  bbox: [number, number, number, number];
}

export interface ImageObject {
  id: PdfObjectId;
  kind: "image";
  pdfRef: PdfRef;
  xObject: string;
  cm: [number, number, number, number, number, number];
  bbox: [number, number, number, number];
}

export interface PathObject {
  id: PdfObjectId;
  kind: "path";
  pdfRef: PdfRef;
  gs?: string;
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

export interface TransformOp {
  op: "transform";
  target: {
    page: number;
    id: PdfObjectId;
  };
  deltaMatrixPt: [number, number, number, number, number, number];
  kind: PageObject["kind"];
}

export interface EditTextOp {
  op: "editText";
  target: {
    page: number;
    id: PdfObjectId;
  };
  text: string;
  fontPref?: {
    preferExisting?: boolean;
    fallbackFamily?: string;
  };
}

export interface SetStyleOp {
  op: "setStyle";
  target: {
    page: number;
    id: PdfObjectId;
  };
  style: {
    colorFill?: [number, number, number];
    colorStroke?: [number, number, number];
    opacityFill?: number;
    opacityStroke?: number;
  };
}

export type PatchOperation = TransformOp | EditTextOp | SetStyleOp;

export interface PatchResponse {
  ok: boolean;
  updatedPdf?: string;
  remap?: Record<string, { pdfRef: PdfRef }>;
}
