export type PdfRef = {
  obj: number;
  gen: number;
};

export type TextObject = {
  id: string;
  kind: "text";
  pdfRef: PdfRef;
  btSpan: { start: number; end: number; streamObj: number };
  Tm: [number, number, number, number, number, number];
  font: { resName: string; size: number; type: string };
  unicode: string;
  glyphs: { gid: number; dx: number; dy: number }[];
  bbox: [number, number, number, number];
};

export type ImageObject = {
  id: string;
  kind: "image";
  pdfRef: PdfRef;
  xObject: string;
  cm: [number, number, number, number, number, number];
  bbox: [number, number, number, number];
};

export type PathObject = {
  id: string;
  kind: "path";
  pdfRef: PdfRef;
  cm: [number, number, number, number, number, number];
  bbox: [number, number, number, number];
};

export type IrObject = TextObject | ImageObject | PathObject;

export type Page = {
  index: number;
  widthPt: number;
  heightPt: number;
  objects: IrObject[];
};

export type DocumentIr = {
  pages: Page[];
};

export type TransformPatch = {
  op: "transform";
  target: { page: number; id: string };
  deltaMatrixPt: [number, number, number, number, number, number];
  kind: "text" | "image" | "path";
};

export type EditTextPatch = {
  op: "editText";
  target: { page: number; id: string };
  text: string;
  fontPref: {
    preferExisting: boolean;
    fallbackFamily: string;
  };
};

export type SetStylePatch = {
  op: "setStyle";
  target: { page: number; id: string };
  style: {
    opacityFill?: number;
    opacityStroke?: number;
    fillColorRgb?: [number, number, number];
    strokeColorRgb?: [number, number, number];
  };
};

export type PatchOperation = TransformPatch | EditTextPatch | SetStylePatch;

export type PatchResponse = {
  ok: boolean;
  updatedPdf?: string;
  remap?: Record<string, { pdfRef: PdfRef }>;
  error?: string;
};
