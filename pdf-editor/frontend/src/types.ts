export interface PdfRef {
  obj: number;
  gen: number;
}

export interface TextObject {
  id: string;
  kind: "text";
  pdfRef: PdfRef;
  Tm: [number, number, number, number, number, number];
  font: {
    resName: string;
    size: number;
    type: string;
  };
  unicode: string;
  bbox: [number, number, number, number];
}

export interface ImageObject {
  id: string;
  kind: "image";
  pdfRef: PdfRef;
  cm: [number, number, number, number, number, number];
  bbox: [number, number, number, number];
}

export type IrObject = TextObject | ImageObject;

export interface PageIR {
  index: number;
  widthPt: number;
  heightPt: number;
  objects: IrObject[];
}

export interface DocumentIR {
  pages: PageIR[];
}

export interface TransformPatch {
  op: "transform";
  target: { page: number; id: string };
  deltaMatrixPt: [number, number, number, number, number, number];
  kind: "text" | "image";
}

export type PatchOp = TransformPatch;
