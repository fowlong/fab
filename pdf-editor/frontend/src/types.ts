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

export interface TextRun {
  id: string;
  kind: "text";
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
  kind: "image" | "path";
  pdfRef: PdfRef;
  xObject?: string;
  cm: PdfMatrix;
  bbox: [number, number, number, number];
}

export type IrObject = TextRun | ImageObject;

export interface IrPage {
  index: number;
  widthPt: number;
  heightPt: number;
  objects: IrObject[];
}

export interface IrDocument {
  pages: IrPage[];
}

export interface TransformPatchOp {
  op: "transform";
  target: { page: number; id: string };
  deltaMatrixPt: PdfMatrix;
  kind: "text" | "image" | "path";
}

export interface EditTextPatchOp {
  op: "editText";
  target: { page: number; id: string };
  text: string;
  fontPref?: {
    preferExisting?: boolean;
    fallbackFamily?: string;
  };
}

export interface SetStylePatchOp {
  op: "setStyle";
  target: { page: number; id: string };
  style: {
    opacityFill?: number;
    opacityStroke?: number;
    fillColor?: [number, number, number];
    strokeColor?: [number, number, number];
  };
}

export type PatchOp = TransformPatchOp | EditTextPatchOp | SetStylePatchOp;

export interface PatchResponse {
  ok: boolean;
  updatedPdf?: string;
  remap?: Record<string, { pdfRef: PdfRef }>;
  message?: string;
}
