export interface PdfRef {
  obj: number;
  gen: number;
}

export type ObjectKind = 'text' | 'image' | 'path';

export interface IrObjectBase {
  id: string;
  kind: ObjectKind;
  pdfRef: PdfRef;
  bbox: [number, number, number, number];
}

export interface TextObject extends IrObjectBase {
  kind: 'text';
  btSpan: { start: number; end: number; streamObj: number };
  Tm: [number, number, number, number, number, number];
  font: {
    resName: string;
    size: number;
    type: string;
  };
  unicode: string;
}

export interface ImageObject extends IrObjectBase {
  kind: 'image';
  xObject: string;
  cm: [number, number, number, number, number, number];
}

export interface PathObject extends IrObjectBase {
  kind: 'path';
  cm: [number, number, number, number, number, number];
}

export type IrObject = TextObject | ImageObject | PathObject;

export interface PageIR {
  index: number;
  widthPt: number;
  heightPt: number;
  objects: IrObject[];
}

export interface DocumentIR {
  pages: PageIR[];
}

export interface TransformPatch {
  op: 'transform';
  target: { page: number; id: string };
  deltaMatrixPt: [number, number, number, number, number, number];
  kind: ObjectKind;
}

export interface EditTextPatch {
  op: 'editText';
  target: { page: number; id: string };
  text: string;
  fontPref: { preferExisting: boolean; fallbackFamily: string };
}

export interface SetStylePatch {
  op: 'setStyle';
  target: { page: number; id: string };
  style: { opacityFill?: number };
}

export type PatchOp = TransformPatch | EditTextPatch | SetStylePatch;

export interface PatchResponse {
  ok: boolean;
  updatedPdf?: string;
  remap?: Record<string, { pdfRef: PdfRef }>;
}
