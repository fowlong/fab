export type PdfRef = {
  obj: number;
  gen: number;
};

export type TextGlyph = {
  gid: number;
  dx: number;
  dy: number;
};

export type TextRun = {
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
  bbox: [number, number, number, number];
  cm: [number, number, number, number, number, number];
};

export type PageObject = TextRun | ImageObject | PathObject;

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
  target: {
    page: number;
    id: string;
  };
  deltaMatrixPt: [number, number, number, number, number, number];
  kind: PageObject['kind'];
};

export type EditTextPatch = {
  op: 'editText';
  target: {
    page: number;
    id: string;
  };
  text: string;
  fontPref: {
    preferExisting: boolean;
    fallbackFamily: string;
  };
};

export type SetStylePatch = {
  op: 'setStyle';
  target: {
    page: number;
    id: string;
  };
  style: {
    opacityFill?: number;
    opacityStroke?: number;
    fillColor?: string;
    strokeColor?: string;
  };
};

export type PatchOp = TransformPatch | EditTextPatch | SetStylePatch;

export type PatchResponse = {
  ok: boolean;
  updatedPdf?: string;
  remap?: Record<string, { pdfRef: PdfRef }>;
  error?: string;
};
