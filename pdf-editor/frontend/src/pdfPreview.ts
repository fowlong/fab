import { GlobalWorkerOptions, getDocument, type PDFDocumentProxy } from 'pdfjs-dist';

GlobalWorkerOptions.workerSrc = 'https://cdnjs.cloudflare.com/ajax/libs/pdf.js/3.11.174/pdf.worker.min.js';

export type RenderedPage = {
  pageIndex: number;
  canvas: HTMLCanvasElement;
  viewportWidth: number;
  viewportHeight: number;
};

export async function renderDocument(
  fileData: Uint8Array,
  resolveHost: (pageIndex: number) => HTMLElement
): Promise<RenderedPage[]> {
  const pdf: PDFDocumentProxy = await getDocument({ data: fileData }).promise;
  const pages: RenderedPage[] = [];

  for (let pageIndex = 0; pageIndex < pdf.numPages; pageIndex++) {
    const page = await pdf.getPage(pageIndex + 1);
    const viewport = page.getViewport({ scale: 1 });
    const canvas = document.createElement('canvas');
    const context = canvas.getContext('2d');
    if (!context) {
      throw new Error('Canvas context unavailable');
    }
    canvas.width = viewport.width;
    canvas.height = viewport.height;
    canvas.className = 'pdf-preview-canvas';

    await page.render({ canvasContext: context, viewport }).promise;

    const host = resolveHost(pageIndex);
    host.textContent = '';
    host.appendChild(canvas);
    pages.push({
      pageIndex,
      canvas,
      viewportWidth: viewport.width,
      viewportHeight: viewport.height
    });
  }

  return pages;
}
