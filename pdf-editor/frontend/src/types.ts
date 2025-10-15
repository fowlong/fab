import type { fabric } from 'fabric';

export interface PdfRef {
  obj: number;
  gen: number;
}

export type IrObject =
  | {
      id: string;
      kind: 'text';
      pdfRef: PdfRef;
      btSpan: { start: number; end: number; streamObj: number };
      Tm: [number, number, number, number, number, number];
      font: { resName: string; size: number; type: string };
      unicode: string;
      glyphs: { gid: number; dx: number; dy: number }[];
      bbox: [number, number, number, number];
    }
  | {
      id: string;
      kind: 'image';
      pdfRef: PdfRef;
      xObject: string;
      cm: [number, number, number, number, number, number];
      bbox: [number, number, number, number];
    }
  | {
      id: string;
      kind: 'path';
      pdfRef: PdfRef;
      cm: [number, number, number, number, number, number];
      bbox: [number, number, number, number];
    };

export interface IrPage {
  index: number;
  widthPt: number;
  heightPt: number;
  objects: IrObject[];
}

export interface IrDocument {
  pages: IrPage[];
}

export interface EditorContext {
  docId: string | null;
  pages: IrPage[];
  overlayByPage: Map<number, fabric.Canvas>;
  setStatus(message: string): void;
}

export interface PdfPreviewPage {
  pageIndex: number;
  canvas: HTMLCanvasElement;
  container: HTMLDivElement;
}

export interface PdfPreview {
  pages: PdfPreviewPage[];
}

export type PatchOperation =
  | {
      op: 'transform';
      target: { page: number; id: string };
      deltaMatrixPt: [number, number, number, number, number, number];
      kind: 'text' | 'image' | 'path';
    }
  | {
    op: 'editText';
    target: { page: number; id: string };
    text: string;
    fontPref: {
      preferExisting: boolean;
      fallbackFamily: string;
    };
  };

export interface PatchResponse {
  ok: boolean;
  updatedPdf?: string;
  remap?: Record<string, { pdfRef: PdfRef }>;
  error?: string;
}
