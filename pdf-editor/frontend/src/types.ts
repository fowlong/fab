export type Matrix = [number, number, number, number, number, number];

export interface PdfRef {
  obj: number;
  gen: number;
}

export interface BtSpan {
  start: number;
  end: number;
  streamObj: number;
}

export interface FontInfo {
  resName: string;
  size: number;
  type?: string;
}

export interface GlyphInfo {
  gid: number;
  dx?: number;
  dy?: number;
}

export interface TextObject {
  id: string;
  kind: 'text';
  pdfRef: PdfRef;
  bbox: [number, number, number, number];
  btSpan?: BtSpan;
  Tm?: Matrix;
  font?: FontInfo;
  unicode?: string;
  glyphs?: GlyphInfo[];
}

export interface ImageObject {
  id: string;
  kind: 'image';
  pdfRef: PdfRef;
  bbox: [number, number, number, number];
  xObject?: string;
  cm?: Matrix;
}

export interface PathObject {
  id: string;
  kind: 'path';
  pdfRef: PdfRef;
  bbox: [number, number, number, number];
  cm?: Matrix;
}

export type IrObject = TextObject | ImageObject | PathObject;

export interface PageIr {
  index: number;
  widthPt: number;
  heightPt: number;
  objects: IrObject[];
}

export interface DocumentIr {
  pages: PageIr[];
}

export interface PatchTarget {
  page: number;
  id: string;
}

export type PatchOperation =
  | {
      op: 'transform';
      target: PatchTarget;
      deltaMatrixPt: Matrix;
      kind: 'text' | 'image' | 'path';
    }
  | {
      op: 'editText';
      target: PatchTarget;
      text: string;
      fontPref?: {
        preferExisting?: boolean;
        fallbackFamily?: string;
      };
    }
  | {
      op: 'setStyle';
      target: PatchTarget;
      style: {
        fill?: [number, number, number];
        stroke?: [number, number, number];
        opacityFill?: number;
        opacityStroke?: number;
      };
    };

export interface PatchResponse {
  ok: boolean;
  updatedPdf: string;
  remap: Record<string, unknown>;
}
