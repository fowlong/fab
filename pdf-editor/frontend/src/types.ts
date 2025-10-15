export interface DocumentIR {
  docId: string;
  pdfUrl?: string;
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
  Tm: [number, number, number, number, number, number];
  font: {
    resName: string;
    size: number;
    type: string;
  };
  unicode: string;
}

export interface ImageObjectIR extends BaseObjectIR {
  kind: "image";
  pdfRef: PdfRef;
  xObject: string;
  cm: [number, number, number, number, number, number];
}

export interface PathObjectIR extends BaseObjectIR {
  kind: "path";
  pdfRef: PdfRef;
  cm: [number, number, number, number, number, number];
}

export interface PdfRef {
  obj: number;
  gen: number;
}

export type PatchOperation = TransformPatch | TextEditPatch | StylePatch;

export interface TransformPatch {
  op: "transform";
  target: PatchTarget;
  deltaMatrixPt: [number, number, number, number, number, number];
  kind: "text" | "image" | "path";
}

export interface TextEditPatch {
  op: "editText";
  target: PatchTarget;
  text: string;
  fontPref?: {
    preferExisting: boolean;
    fallbackFamily?: string;
  };
}

export interface StylePatch {
  op: "setStyle";
  target: PatchTarget;
  style: {
    opacityFill?: number;
    opacityStroke?: number;
    fillColor?: [number, number, number];
    strokeColor?: [number, number, number];
  };
}

export interface PatchTarget {
  page: number;
  id: string;
}
