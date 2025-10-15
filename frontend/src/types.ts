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
  kind: 'text';
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
  kind: 'image';
  pdfRef: PdfRef;
  xObject: string;
  cm: [number, number, number, number, number, number];
  bbox: [number, number, number, number];
};

export type PathObject = {
  id: string;
  kind: 'path';
  pdfRef: PdfRef;
  operations: string[];
  cm: [number, number, number, number, number, number];
  bbox: [number, number, number, number];
};

export type PageObject = TextObject | ImageObject | PathObject;

export type PageIR = {
  index: number;
  widthPt: number;
  heightPt: number;
  objects: PageObject[];
};

export type DocumentIR = {
  pages: PageIR[];
};

export type TransformPatch = {
  op: 'transform';
  target: PatchTarget;
  deltaMatrixPt: [number, number, number, number, number, number];
  kind: 'text' | 'image' | 'path';
};

export type EditTextPatch = {
  op: 'editText';
  target: PatchTarget;
  text: string;
  fontPref?: {
    preferExisting?: boolean;
    fallbackFamily?: string;
  };
};

export type StylePatch = {
  op: 'setStyle';
  target: PatchTarget;
  style: {
    fillColor?: [number, number, number];
    strokeColor?: [number, number, number];
    opacityFill?: number;
    opacityStroke?: number;
  };
};

export type PatchOperation = TransformPatch | EditTextPatch | StylePatch;

export type PatchTarget = {
  page: number;
  id: string;
};

export type PatchResponse = {
  ok: boolean;
  updatedPdf?: string;
  remap?: Record<string, { pdfRef: PdfRef }>;
  message?: string;
};
