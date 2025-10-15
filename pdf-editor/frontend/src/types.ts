export type PdfRef = {
  obj: number;
  gen: number;
};

export type TextObject = {
  id: string;
  kind: 'text';
  pdfRef?: PdfRef;
  btSpan?: { start: number; end: number; streamObj: number };
  Tm?: [number, number, number, number, number, number];
  font?: { resName: string; size: number; type: string };
  unicode: string;
  glyphs?: { gid: number; dx: number; dy: number }[];
  bbox: [number, number, number, number];
  transform?: [number, number, number, number, number, number];
};

export type ImageObject = {
  id: string;
  kind: 'image';
  pdfRef?: PdfRef;
  xObject?: string;
  cm?: [number, number, number, number, number, number];
  bbox: [number, number, number, number];
  transform?: [number, number, number, number, number, number];
};

export type PathObject = {
  id: string;
  kind: 'path';
  pdfRef?: PdfRef;
  bbox: [number, number, number, number];
  transform?: [number, number, number, number, number, number];
};

export type PageIR = {
  index: number;
  widthPt: number;
  heightPt: number;
  objects: (TextObject | ImageObject | PathObject)[];
};

export type DocumentIR = {
  docId: string;
  sourceUrl?: string;
  pages: PageIR[];
};

export type TransformOperation = {
  op: 'transform';
  kind: 'text' | 'image' | 'path';
  target: { page: number; id: string };
  deltaMatrixPt: [number, number, number, number, number, number];
};

export type EditTextOperation = {
  op: 'editText';
  target: { page: number; id: string };
  text: string;
  fontPref?: { preferExisting?: boolean; fallbackFamily?: string };
};

export type SetStyleOperation = {
  op: 'setStyle';
  target: { page: number; id: string };
  style: { colorFill?: [number, number, number]; opacityFill?: number };
};

export type PatchOperation = TransformOperation | EditTextOperation | SetStyleOperation;

export type PatchResponse = {
  ok: boolean;
  updatedPdf?: string;
  remap?: Record<string, { pdfRef: PdfRef }>;
  message?: string;
};
