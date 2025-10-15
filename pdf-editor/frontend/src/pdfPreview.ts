import { getDocument, GlobalWorkerOptions, type PDFDocumentProxy } from 'pdfjs-dist';
import 'pdfjs-dist/build/pdf.worker.mjs';

export interface PreviewPage {
  canvas: HTMLCanvasElement;
  context: CanvasRenderingContext2D;
}

GlobalWorkerOptions.workerSrc = 'pdfjs-dist/build/pdf.worker.mjs';

export async function renderPdf(
  container: HTMLElement,
  pdfData: ArrayBuffer
): Promise<PreviewPage[]> {
  container.innerHTML = '';
  const loadingTask = getDocument({ data: pdfData });
  const doc: PDFDocumentProxy = await loadingTask.promise;
  const pages: PreviewPage[] = [];

  for (let pageNum = 1; pageNum <= doc.numPages; pageNum += 1) {
    const page = await doc.getPage(pageNum);
    const viewport = page.getViewport({ scale: 1.5 });
    const canvas = document.createElement('canvas');
    canvas.width = viewport.width;
    canvas.height = viewport.height;
    const context = canvas.getContext('2d');
    if (!context) continue;
    await page.render({ canvasContext: context, viewport }).promise;
    container.appendChild(canvas);
    pages.push({ canvas, context });
  }

  return pages;
}
