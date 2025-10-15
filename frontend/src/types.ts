export interface PdfRef {
  obj: number;
  gen: number;
}

export type Matrix = [number, number, number, number, number, number];
export type BoundingBox = [number, number, number, number];

export interface StreamSpan {
  start: number;
  end: number;
  streamObj: number;
}

export interface Glyph {
  gid: number;
  dx: number;
  dy: number;
}

export interface TextFont {
  resName: string;
  size: number;
  type: string;
}

export interface TextObject {
  id: string;
  pdfRef: PdfRef;
  btSpan: StreamSpan;
  Tm: Matrix;
  font: TextFont;
  unicode: string;
  glyphs: Glyph[];
  bbox: BoundingBox;
}

export interface ImageObject {
  id: string;
  pdfRef: PdfRef;
  xObject: string;
  cm: Matrix;
  bbox: BoundingBox;
}

export interface PathObject {
  id: string;
  pdfRef: PdfRef;
  cm: Matrix;
  bbox: BoundingBox;
}

export type IrObject =
  | ({ kind: "text" } & TextObject)
  | ({ kind: "image" } & ImageObject)
  | ({ kind: "path" } & PathObject);

export interface PageIr {
  index: number;
  widthPt: number;
  heightPt: number;
  objects: IrObject[];
}

export interface DocumentIr {
  pages: PageIr[];
}

export interface TransformTarget {
  page: number;
  id: string;
}

export type TransformKind = "text" | "image" | "path";

export interface TransformOp {
  deltaMatrixPt: Matrix;
  kind: TransformKind;
}

export interface FontPreference {
  preferExisting: boolean;
  fallbackFamily?: string;
}

export interface StylePatch {
  fillColor?: [number, number, number];
  strokeColor?: [number, number, number];
  opacityFill?: number;
  opacityStroke?: number;
}

export type PatchOperation =
  | { op: "transform"; target: TransformTarget; deltaMatrixPt: Matrix; kind: TransformKind }
  | { op: "editText"; target: TransformTarget; text: string; fontPref?: FontPreference }
  | { op: "setStyle"; target: TransformTarget; style: StylePatch };

export interface PatchResponse {
  ok: boolean;
  updatedPdf?: string;
  remap?: Record<string, unknown>;
}
