export interface PdfRef {
  obj: number;
  gen: number;
}

export interface TextObject {
  id: string;
  kind: "text";
  pdfRef: PdfRef;
  btSpan: { start: number; end: number; streamObj: number };
  Tm: [number, number, number, number, number, number];
  font: { resName: string; size: number; type: string };
  unicode: string;
  glyphs: { gid: number; dx: number; dy: number }[];
  bbox: [number, number, number, number];
}

export interface ImageObject {
  id: string;
  kind: "image";
  pdfRef: PdfRef;
  xObject: string;
  cm: [number, number, number, number, number, number];
  bbox: [number, number, number, number];
}

export interface PathObject {
  id: string;
  kind: "path";
  pdfRef: PdfRef;
  cm: [number, number, number, number, number, number];
  bbox: [number, number, number, number];
}

export type IRObject = TextObject | ImageObject | PathObject;

export interface IRPage {
  index: number;
  widthPt: number;
  heightPt: number;
  objects: IRObject[];
}

export interface IRDocument {
  pages: IRPage[];
}

export interface TransformPatch {
  op: "transform";
  target: { page: number; id: string };
  kind: IRObject["kind"];
  deltaMatrixPt: [number, number, number, number, number, number];
}

export type PatchOperation = TransformPatch;

export interface EditorState {
  docId: string | null;
  pages: IRPage[];
  fabricOverlays: Map<number, { canvas: fabric.Canvas; objectIndex: Map<string, fabric.Object> }>;
}
