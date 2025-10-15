export type PdfRef = { obj: number; gen: number };

export type TextGlyph = {
  gid: number;
  dx?: number;
  dy?: number;
};

export interface TextObject {
  id: string;
  kind: 'text';
  pdfRef: PdfRef;
  btSpan?: { start: number; end: number; streamObj: number };
  Tm: [number, number, number, number, number, number];
  font: {
    resName: string;
    size: number;
    type?: string;
  };
  unicode: string;
  glyphs?: TextGlyph[];
  bbox: [number, number, number, number];
}

export interface ImageObject {
  id: string;
  kind: 'image';
  pdfRef: PdfRef;
  xObject: string;
  cm: [number, number, number, number, number, number];
  bbox: [number, number, number, number];
}

export interface PathObject {
  id: string;
  kind: 'path';
  pdfRef: PdfRef;
  cm?: [number, number, number, number, number, number];
  bbox: [number, number, number, number];
}

export type PageObject = TextObject | ImageObject | PathObject;

export interface PageIR {
  index: number;
  widthPt: number;
  heightPt: number;
  objects: PageObject[];
}

export interface DocumentIR {
  pages: PageIR[];
}

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
    fillColor?: string;
    strokeColor?: string;
    opacityFill?: number;
    opacityStroke?: number;
  };
};

export type PatchOp = TransformPatch | EditTextPatch | StylePatch;

export type PatchResponse = {
  ok: boolean;
  message?: string;
  updatedPdf?: string;
  remap?: Record<string, { pdfRef: PdfRef }>;
};

export interface PatchTarget {
  page: number;
  id: string;
}

export type FabricMeta = {
  id: string;
  page: number;
  baseMatrixPx: [number, number, number, number, number, number];
};

export interface ToastMessage {
  id: string;
  text: string;
  tone: 'info' | 'success' | 'error';
}
