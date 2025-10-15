import { GlobalWorkerOptions, getDocument, type PDFDocumentProxy } from 'pdfjs-dist';
import 'pdfjs-dist/build/pdf.worker?worker';

GlobalWorkerOptions.workerSrc = new URL(
  'pdfjs-dist/build/pdf.worker.min.js',
  import.meta.url
).toString();

export interface PdfPreviewPage {
  canvas: HTMLCanvasElement;
  container: HTMLDivElement;
}

export async function renderPdf(
  data: Uint8Array,
  mount: HTMLElement
): Promise<PdfPreviewPage[]> {
  mount.innerHTML = '';
  const loadingTask = getDocument({ data });
  const pdf: PDFDocumentProxy = await loadingTask.promise;
  const pages: PdfPreviewPage[] = [];

  for (let pageNumber = 1; pageNumber <= pdf.numPages; pageNumber++) {
    const page = await pdf.getPage(pageNumber);
    const viewport = page.getViewport({ scale: 1.5 });
    const canvas = document.createElement('canvas');
    const context = canvas.getContext('2d');
    if (!context) {
      throw new Error('Unable to get canvas context');
    }
    canvas.width = viewport.width;
    canvas.height = viewport.height;

    const renderTask = page.render({ canvasContext: context, viewport });
    await renderTask.promise;

    const container = document.createElement('div');
    container.className = 'page-container';
    container.style.position = 'relative';
    container.style.width = `${viewport.width}px`;
    container.style.height = `${viewport.height}px`;
    container.append(canvas);
    mount.append(container);

    pages.push({ canvas, container });
  }

  return pages;
}
