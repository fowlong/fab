export interface PdfRef {
  obj: number;
  gen: number;
}

export interface TextGlyph {
  gid: number;
  dx: number;
  dy: number;
}

export interface TextObject {
  id: string;
  kind: "text";
  pdfRef: PdfRef;
  btSpan: { start: number; end: number; streamObj: number };
  Tm: [number, number, number, number, number, number];
  font: { resName: string; size: number; type: string };
  unicode: string;
  glyphs: TextGlyph[];
  bbox: [number, number, number, number];
}

export interface ImageObject {
  id: string;
  kind: "image";
  pdfRef: PdfRef;
  xObject: string;
  cm: [number, number, number, number, number, number];
  bbox: [number, number, number, number];
}

export interface PathObject {
  id: string;
  kind: "path";
  pdfRef: PdfRef;
  cm: [number, number, number, number, number, number];
  bbox: [number, number, number, number];
}

export type PageObject = TextObject | ImageObject | PathObject;

export interface IrPage {
  index: number;
  widthPt: number;
  heightPt: number;
  objects: PageObject[];
}

export interface IrDocument {
  pages: IrPage[];
}

export interface ApiClient {
  openDocument(file: File): Promise<string>;
  loadIr(docId: string): Promise<IrDocument>;
  patch(docId: string, ops: unknown[]): Promise<void>;
  download(docId: string): Promise<Blob>;
}

export interface FabricOverlayOptions {
  pages: IrPage[];
  preview: import("./pdfPreview").PdfPreviewContext;
  api: ApiClient;
  docId: string;
  onStatus?: (message: string) => void;
}
