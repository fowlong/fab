import type { PdfPageView } from "pdfjs-dist/web/pdf_viewer";

export interface PreviewHandle {
  container: HTMLElement;
  pages: PdfPageView[];
}

let currentHandle: PreviewHandle | null = null;

export function initialisePreview(container: HTMLElement) {
  currentHandle = {
    container,
    pages: []
  };
}

export function getPreviewHandle(): PreviewHandle | null {
  return currentHandle;
}
