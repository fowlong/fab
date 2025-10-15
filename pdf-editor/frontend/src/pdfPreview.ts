import { GlobalWorkerOptions, getDocument, type PDFDocumentProxy } from 'pdfjs-dist';

GlobalWorkerOptions.workerSrc = new URL(
  'pdfjs-dist/build/pdf.worker.min.mjs',
  import.meta.url
).toString();

export async function renderPdfInto(
  container: HTMLElement,
  pdfData: Uint8Array
): Promise<PDFDocumentProxy> {
  const loadingTask = getDocument({ data: pdfData });
  const pdf = await loadingTask.promise;

  container.replaceChildren();
  for (let i = 1; i <= pdf.numPages; i += 1) {
    const page = await pdf.getPage(i);
    const viewport = page.getViewport({ scale: 1 });
    const canvas = document.createElement('canvas');
    canvas.width = viewport.width;
    canvas.height = viewport.height;
    canvas.className = 'pdf-preview-canvas';
    const ctx = canvas.getContext('2d');
    if (!ctx) {
      throw new Error('Unable to get 2D context');
    }
    await page.render({ canvasContext: ctx, viewport }).promise;
    container.appendChild(canvas);
  }
  return pdf;
}
