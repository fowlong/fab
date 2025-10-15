import type { Matrix } from "./coords";

export interface PdfRef {
  obj: number;
  gen: number;
}

export interface IrTextGlyph {
  gid: number;
  dx: number;
  dy: number;
}

interface IrBase {
  id: string;
  page: number;
  pdfRef: PdfRef;
  bbox: [number, number, number, number];
}

export interface IrTextRun extends IrBase {
  kind: "text";
  btSpan?: {
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
  glyphs: IrTextGlyph[];
}

export interface IrImage extends IrBase {
  kind: "image";
  xObject: string;
  cm: Matrix;
}

export interface IrPath extends IrBase {
  kind: "path";
  cm: Matrix;
}

export type IrObject = IrTextRun | IrImage | IrPath;

export interface IrPage {
  index: number;
  widthPt: number;
  heightPt: number;
  objects: IrObject[];
}

export interface IrDocument {
  pages: IrPage[];
}

export interface TransformPatch {
  op: "transform";
  target: { page: number; id: string };
  deltaMatrixPt: Matrix;
  kind: "text" | "image" | "path";
}

export interface EditTextPatch {
  op: "editText";
  target: { page: number; id: string };
  text: string;
  fontPref?: {
    preferExisting?: boolean;
    fallbackFamily?: string;
  };
}

export interface SetStylePatch {
  op: "setStyle";
  target: { page: number; id: string };
  style: {
    colorFill?: [number, number, number];
    colorStroke?: [number, number, number];
    opacityFill?: number;
    opacityStroke?: number;
  };
}

export type PatchOperation = TransformPatch | EditTextPatch | SetStylePatch;

export interface PatchResponse {
  ok: boolean;
  updatedPdf?: string;
  remap?: Record<string, { pdfRef: PdfRef }>;
}

export interface FabricObjectMeta {
  id: string;
  page: number;
  pageHeightPt: number;
  initialMatrix: Matrix;
}
