export type Matrix = [number, number, number, number, number, number];

export type Span = {
  streamObj: number;
  start: number;
  end: number;
};

export type FontSummary = {
  resName: string;
  size: number;
};

export type TextObject = {
  kind: 'text';
  id: string;
  btSpan: Span;
  Tm: Matrix;
  font: FontSummary;
  bbox: [number, number, number, number];
};

export type ImageObject = {
  kind: 'image';
  id: string;
  xObject: string;
  cm: Matrix;
  bbox: [number, number, number, number];
};

export type PageObject = TextObject | ImageObject;

export type PageIR = {
  index: number;
  widthPt: number;
  heightPt: number;
  objects: PageObject[];
};

export type DocumentIR = {
  pages: PageIR[];
};

export type PatchTarget = {
  page: number;
  id: string;
};

export type TransformPatch = {
  op: 'transform';
  target: PatchTarget;
  deltaMatrixPt: Matrix;
  kind: 'text' | 'image';
};

export type PatchOperation = TransformPatch;

export type PatchResponse = {
  ok: boolean;
  updatedPdf?: string;
  remap?: Record<string, unknown>;
};
