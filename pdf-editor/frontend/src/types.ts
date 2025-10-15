export type PdfRef = {
  obj: number;
  gen: number;
};

export type TextGlyph = {
  gid: number;
  dx: number;
  dy: number;
};

export type TextObject = {
  id: string;
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
  glyphs: TextGlyph[];
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
  bbox: [number, number, number, number];
  cm?: [number, number, number, number, number, number];
};

export type PageObject = TextObject | ImageObject | PathObject;

export type Page = {
  index: number;
  widthPt: number;
  heightPt: number;
  objects: PageObject[];
};

export type DocumentIR = {
  pages: Page[];
};

export type PatchTransform = {
  op: "transform";
  target: { page: number; id: string };
  deltaMatrixPt: [number, number, number, number, number, number];
  kind: "text" | "image" | "path";
};

export type PatchEditText = {
  op: "editText";
  target: { page: number; id: string };
  text: string;
  fontPref: { preferExisting: boolean; fallbackFamily?: string };
};

export type PatchSetStyle = {
  op: "setStyle";
  target: { page: number; id: string };
  style: { colorFill?: [number, number, number]; opacityFill?: number };
};

export type PatchOp = PatchTransform | PatchEditText | PatchSetStyle;

export type PatchResponse = {
  ok: boolean;
  updatedPdf?: string;
  remap?: Record<string, { pdfRef: PdfRef }>;
};
