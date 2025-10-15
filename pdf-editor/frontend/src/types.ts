export interface DocumentIR {
  docId?: string;
  pdfDataUrl?: string;
  pages: PageIR[];
}

export interface PageIR {
  index: number;
  widthPt: number;
  heightPt: number;
  widthPx: number;
  heightPx: number;
  objects: PageObject[];
}

export type PageObject = TextObject | ImageObject | PathObject;

export interface BaseObject {
  id: string;
  kind: "text" | "image" | "path";
  bboxPt: [number, number, number, number];
  bboxPx: [number, number, number, number];
}

export interface TextObject extends BaseObject {
  kind: "text";
  Tm: [number, number, number, number, number, number];
  unicode: string;
  font: {
    resName: string;
    size: number;
    type: string;
  };
}

export interface ImageObject extends BaseObject {
  kind: "image";
  cm: [number, number, number, number, number, number];
}

export interface PathObject extends BaseObject {
  kind: "path";
  cm: [number, number, number, number, number, number];
}

export interface PatchTransformOp {
  op: "transform";
  target: { page: number; id: string };
  deltaMatrixPt: [number, number, number, number, number, number];
  kind: PageObject["kind"];
}

export type PatchOp = PatchTransformOp;
