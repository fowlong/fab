import type { PDFDocumentProxy, PDFPageProxy } from 'pdfjs-dist';
import { getDocument } from 'pdfjs-dist';

export async function loadPdf(url: string): Promise<PDFDocumentProxy> {
  const loadingTask = getDocument(url);
  return loadingTask.promise;
}

export async function renderPage(
  pdf: PDFDocumentProxy,
  pageNumber: number,
  canvas: HTMLCanvasElement
): Promise<void> {
  const page: PDFPageProxy = await pdf.getPage(pageNumber);
  const viewport = page.getViewport({ scale: 1.0 });
  const context = canvas.getContext('2d');
  if (!context) {
    throw new Error('Canvas 2D context not available');
  }
  canvas.width = viewport.width;
  canvas.height = viewport.height;
  await page.render({ canvasContext: context, viewport }).promise;
}
