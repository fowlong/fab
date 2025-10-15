import { getDocument, GlobalWorkerOptions, type PDFDocumentProxy } from 'pdfjs-dist';
import type { PageIR } from './types';

GlobalWorkerOptions.workerSrc = new URL('pdf.worker.min.js', import.meta.url).toString();

export async function loadPdf(data: ArrayBuffer): Promise<PDFDocumentProxy> {
  const task = getDocument({ data });
  return task.promise;
}

export async function renderPage(
  pdf: PDFDocumentProxy,
  pageIR: PageIR,
  canvas: HTMLCanvasElement,
): Promise<void> {
  const page = await pdf.getPage(pageIR.index + 1);
  const viewport = page.getViewport({ scale: 1 });
  canvas.width = viewport.width;
  canvas.height = viewport.height;
  const ctx = canvas.getContext('2d');
  if (!ctx) throw new Error('Failed to get 2D context');
  await page.render({ canvasContext: ctx, viewport }).promise;
}
