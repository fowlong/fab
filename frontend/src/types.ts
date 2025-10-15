export interface DocumentIR {
  pages: PageIR[];
}

export interface PageIR {
  index: number;
  widthPt: number;
  heightPt: number;
  objects: IrObject[];
}

export type IrObject = TextObject | ImageObject | PathObject;

export interface BaseObject {
  id: string;
  kind: "text" | "image" | "path";
  bbox?: [number, number, number, number];
}

export interface TextObject extends BaseObject {
  kind: "text";
  Tm: [number, number, number, number, number, number];
  font: {
    resName: string;
    size: number;
    type: string;
  };
  unicode: string;
}

export interface ImageObject extends BaseObject {
  kind: "image";
  cm: [number, number, number, number, number, number];
}

export interface PathObject extends BaseObject {
  kind: "path";
  cm: [number, number, number, number, number, number];
}

export type PatchOperation = TransformOp | EditTextOp | SetStyleOp;

export interface TransformOp {
  op: "transform";
  kind: IrObject["kind"];
  target: PatchTarget;
  deltaMatrixPt: [number, number, number, number, number, number];
}

export interface EditTextOp {
  op: "editText";
  target: PatchTarget;
  text: string;
  fontPref?: {
    preferExisting?: boolean;
    fallbackFamily?: string;
  };
}

export interface SetStyleOp {
  op: "setStyle";
  target: PatchTarget;
  style: Record<string, unknown>;
}

export interface PatchTarget {
  page: number;
  id: string;
}

export interface PatchResponse {
  ok: boolean;
  updatedPdf: string;
  remap?: Record<string, { pdfRef: { obj: number; gen: number } }>;
}
