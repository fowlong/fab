import { GlobalWorkerOptions, getDocument, type PDFDocumentProxy } from 'pdfjs-dist';

GlobalWorkerOptions.workerSrc = new URL(
  'pdfjs-dist/build/pdf.worker.min.mjs',
  import.meta.url
).toString();

export interface RenderedPage {
  pageNumber: number;
  canvas: HTMLCanvasElement;
  widthPx: number;
  heightPx: number;
  scale: number;
}

export async function renderPdf(buffer: Uint8Array, container: HTMLElement): Promise<RenderedPage[]> {
  container.innerHTML = '';
  const pdf: PDFDocumentProxy = await getDocument({ data: buffer }).promise;
  const pages: RenderedPage[] = [];

  for (let i = 1; i <= pdf.numPages; i += 1) {
    const page = await pdf.getPage(i);
    const scale = 1.5;
    const viewport = page.getViewport({ scale });

    const canvas = document.createElement('canvas');
    const context = canvas.getContext('2d');
    if (!context) {
      throw new Error('Failed to create canvas context');
    }

    canvas.width = viewport.width;
    canvas.height = viewport.height;
    canvas.className = 'pdf-canvas';

    await page.render({ canvasContext: context, viewport }).promise;

    pages.push({
      pageNumber: i,
      canvas,
      widthPx: viewport.width,
      heightPx: viewport.height,
      scale
    });
  }

  return pages;
}
