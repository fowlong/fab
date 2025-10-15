import type { fabric } from "fabric";

export interface PdfOpenResponse {
  docId: string;
}

export interface PdfState {
  pages: IrPage[];
}

export interface IrPage {
  index: number;
  widthPt: number;
  heightPt: number;
  objects: IrObject[];
}

export type IrObject = IrTextObject | IrImageObject | IrPathObject;

export interface IrBaseObject {
  id: string;
  kind: "text" | "image" | "path";
  bbox: [number, number, number, number];
  transform?: [number, number, number, number, number, number];
}

export interface IrTextObject extends IrBaseObject {
  kind: "text";
  unicode: string;
  font: {
    resName: string;
    size: number;
    type: string;
  };
}

export interface IrImageObject extends IrBaseObject {
  kind: "image";
  xObject: string;
}

export interface IrPathObject extends IrBaseObject {
  kind: "path";
}

export interface FabricOverlayHandle {
  canvas: fabric.Canvas;
  pageIndex: number;
  rebuild(objects: IrObject[]): void;
}

export interface FabricObjectDescriptor {
  id: string;
  width: number;
  height: number;
  matrixPx: [number, number, number, number, number, number];
}

export interface EditorState {
  docId: string | null;
  pages: IrPage[];
  overlays: Map<number, FabricOverlayHandle>;
}

export type PatchOperation =
  | {
      op: "transform";
      target: { page: number; id: string };
      deltaMatrixPt: [number, number, number, number, number, number];
      kind: "text" | "image" | "path";
    }
  | {
      op: "editText";
      target: { page: number; id: string };
      text: string;
      fontPref: { preferExisting: boolean; fallbackFamily: string };
    }
  | {
      op: "setStyle";
      target: { page: number; id: string };
      style: Record<string, unknown>;
    };

export interface PatchResponse {
  ok: boolean;
  updatedPdf?: string;
  remap?: Record<string, unknown>;
}
