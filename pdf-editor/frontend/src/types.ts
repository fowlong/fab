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

export type TransformPatchOp = {
  op: 'transform';
  target: PatchTarget;
  deltaMatrixPt: [number, number, number, number, number, number];
  kind: 'text' | 'image' | 'path';
};

export type EditTextPatchOp = {
  op: 'editText';
  target: PatchTarget;
  text: string;
  fontPref?: {
    preferExisting?: boolean;
    fallbackFamily?: string;
  };
};

export type SetStylePatchOp = {
  op: 'setStyle';
  target: PatchTarget;
  style: {
    colorFill?: [number, number, number];
    colorStroke?: [number, number, number];
    opacityFill?: number;
    opacityStroke?: number;
  };
};

export type PatchOp = TransformPatchOp | EditTextPatchOp | SetStylePatchOp;

export type PatchTarget = {
  page: number;
  id: string;
};

export type PatchResponse = {
  ok: boolean;
  updatedPdf?: string;
  remap?: Record<string, { pdfRef: PdfRef }>;
  error?: string;
};
