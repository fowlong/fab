export type Span = {
  start: number;
  end: number;
  streamObj: number;
};

export type FontInfo = {
  resName: string;
  size: number;
};

export type TextObject = {
  id: string;
  kind: 'text';
  btSpan: Span;
  Tm: [number, number, number, number, number, number];
  font: FontInfo;
  bbox: [number, number, number, number];
};

export type ImageObject = {
  id: string;
  kind: 'image';
  xObject: string;
  cm: [number, number, number, number, number, number];
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

export type TransformPatch = {
  op: 'transform';
  target: { page: number; id: string };
  deltaMatrixPt: [number, number, number, number, number, number];
  kind: 'text' | 'image';
};

export type PatchOperation = TransformPatch;

export type PatchResponse = {
  ok: boolean;
  updatedPdf?: string;
  remap?: Record<string, unknown> | null;
};
