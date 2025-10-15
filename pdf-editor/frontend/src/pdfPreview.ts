import { getDocument, GlobalWorkerOptions, type PDFDocumentProxy } from 'pdfjs-dist';
import workerSrc from 'pdfjs-dist/build/pdf.worker?url';

GlobalWorkerOptions.workerSrc = workerSrc;

export type PdfPreviewPage = {
  canvas: HTMLCanvasElement;
  container: HTMLDivElement;
};

export async function renderPdfPreview(
  data: ArrayBuffer,
  root: HTMLElement,
): Promise<{ pages: PdfPreviewPage[]; pdf: PDFDocumentProxy }> {
  root.innerHTML = '';
  const pdf = await getDocument({ data }).promise;
  const pages: PdfPreviewPage[] = [];
  for (let i = 1; i <= pdf.numPages; i += 1) {
    const page = await pdf.getPage(i);
    const viewport = page.getViewport({ scale: 1.5 });
    const canvas = document.createElement('canvas');
    canvas.width = viewport.width;
    canvas.height = viewport.height;
    const ctx = canvas.getContext('2d');
    if (!ctx) {
      throw new Error('Unable to create canvas context');
    }
    await page.render({ canvasContext: ctx, viewport }).promise;
    const wrapper = document.createElement('div');
    wrapper.className = 'pdf-page';
    wrapper.appendChild(canvas);
    root.appendChild(wrapper);
    pages.push({ canvas, container: wrapper });
  }
  return { pages, pdf };
}
