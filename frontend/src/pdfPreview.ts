import type { PageIR } from './types';
import { GlobalWorkerOptions, getDocument, type PDFDocumentProxy } from 'pdfjs-dist';

GlobalWorkerOptions.workerSrc = 'https://cdn.jsdelivr.net/npm/pdfjs-dist@4.2.67/build/pdf.worker.min.js';

export type PdfPreviewOptions = {
  container: HTMLElement;
  pdfData: Uint8Array;
};

export type RenderedPage = {
  page: PageIR;
  canvas: HTMLCanvasElement;
};

export async function renderPdfPreview(options: PdfPreviewOptions): Promise<PDFDocumentProxy> {
  const loadingTask = getDocument({ data: options.pdfData });
  const pdf = await loadingTask.promise;
  options.container.innerHTML = '';
  for (let pageNumber = 1; pageNumber <= pdf.numPages; pageNumber += 1) {
    const page = await pdf.getPage(pageNumber);
    const viewport = page.getViewport({ scale: 1.5 });
    const canvas = document.createElement('canvas');
    const context = canvas.getContext('2d');
    if (!context) {
      throw new Error('Unable to obtain canvas context');
    }
    canvas.height = viewport.height;
    canvas.width = viewport.width;
    options.container.appendChild(canvas);
    await page.render({ canvasContext: context, viewport }).promise;
  }
  return pdf;
}
