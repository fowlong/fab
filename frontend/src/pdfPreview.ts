import { getDocument, GlobalWorkerOptions, type PDFDocumentProxy } from "pdfjs-dist";
import type { PageIR, DocumentMeta } from "./types";

const workerSrc = (globalThis as unknown as { __PDF_WORKER_SRC__?: string }).__PDF_WORKER_SRC__;
if (workerSrc) {
  GlobalWorkerOptions.workerSrc = workerSrc;
}

let currentDoc: PDFDocumentProxy | null = null;

export async function renderPdfPreview(page: PageIR, meta: DocumentMeta) {
  if (!meta.originalPdfBytes) {
    throw new Error("No PDF bytes available for rendering");
  }

  if (!currentDoc) {
    currentDoc = await getDocument({ data: meta.originalPdfBytes }).promise;
  }

  const pdfPage = await currentDoc.getPage(page.index + 1);
  const viewport = pdfPage.getViewport({ scale: 1.5 });
  const canvas = document.createElement("canvas");
  const context = canvas.getContext("2d");
  if (!context) {
    throw new Error("Failed to create 2D context");
  }

  canvas.width = viewport.width;
  canvas.height = viewport.height;
  await pdfPage.render({ canvasContext: context, viewport }).promise;
  return { canvas, viewport };
}

export function resetPdfPreview() {
  currentDoc = null;
}
