import { getDocument, GlobalWorkerOptions, type PDFDocumentProxy } from "pdfjs-dist";
import type { PageIR } from "./types";

GlobalWorkerOptions.workerSrc = `//cdnjs.cloudflare.com/ajax/libs/pdf.js/${pdfjsLibVersion()}/pdf.worker.min.js`;

function pdfjsLibVersion(): string {
  return "4.2.67";
}

export async function loadPdfDocument(pdfUrl: string): Promise<PDFDocumentProxy> {
  const loadingTask = getDocument(pdfUrl);
  return loadingTask.promise;
}

interface PdfPreviewOptions {
  page: PageIR;
  container: HTMLElement;
  pdf: PDFDocumentProxy;
}

export async function initPdfPreview({ page, container, pdf }: PdfPreviewOptions) {
  const canvas = document.createElement("canvas");
  canvas.className = "preview";
  container.appendChild(canvas);

  const pdfPage = await pdf.getPage(page.index + 1);
  const viewport = pdfPage.getViewport({ scale: 1 });

  const context = canvas.getContext("2d");
  if (!context) throw new Error("Failed to get canvas context");

  canvas.width = viewport.width;
  canvas.height = viewport.height;

  await pdfPage.render({ canvasContext: context, viewport }).promise;

  return { canvas };
}
