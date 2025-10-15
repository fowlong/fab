import { GlobalWorkerOptions, getDocument, type PDFDocumentProxy } from 'pdfjs-dist';

GlobalWorkerOptions.workerSrc = new URL(
  'pdfjs-dist/build/pdf.worker.js',
  import.meta.url,
).toString();

export async function renderPage(
  pdfBytes: Uint8Array,
  canvas: HTMLCanvasElement,
  pageIndex: number,
): Promise<void> {
  const loadingTask = getDocument({ data: pdfBytes });
  const pdf: PDFDocumentProxy = await loadingTask.promise;
  const page = await pdf.getPage(pageIndex + 1);
  const viewport = page.getViewport({ scale: window.devicePixelRatio || 1 });
  const context = canvas.getContext('2d');
  if (!context) {
    return;
  }
  canvas.height = viewport.height;
  canvas.width = viewport.width;
  await page.render({ canvasContext: context, viewport }).promise;
}
