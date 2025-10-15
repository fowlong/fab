export type Matrix = [number, number, number, number, number, number];

export type StreamSpan = {
  streamObj: number;
  start: number;
  end: number;
};

export type TextObject = {
  id: string;
  kind: 'text';
  btSpan: StreamSpan;
  Tm: Matrix;
  font: {
    resName: string;
    size: number;
  };
  bbox?: [number, number, number, number];
};

export type ImageObject = {
  id: string;
  kind: 'image';
  xObject: string;
  cm: Matrix;
  bbox?: [number, number, number, number];
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
  target: PatchTarget;
  deltaMatrixPt: Matrix;
  kind: 'text' | 'image';
};

export type PatchOperation = TransformPatch;

export type PatchTarget = {
  page: number;
  id: string;
};

export type PatchResponse = {
  ok: boolean;
  updatedPdf?: string;
  remap?: Record<string, unknown>;
};
