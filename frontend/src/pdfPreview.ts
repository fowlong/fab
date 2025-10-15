import { GlobalWorkerOptions, getDocument, type PDFDocumentProxy } from 'pdfjs-dist';
import workerSrc from 'pdfjs-dist/build/pdf.worker?url';

GlobalWorkerOptions.workerSrc = workerSrc;

export type RenderResult = {
  canvas: HTMLCanvasElement;
  width: number;
  height: number;
  pageHeightPt: number;
};

export class PdfPreview {
  private container: HTMLElement;
  private pdf: PDFDocumentProxy | null = null;

  constructor(container: HTMLElement) {
    this.container = container;
  }

  async renderPage0(data: ArrayBuffer): Promise<RenderResult> {
    this.reset();
    this.pdf = await getDocument({ data }).promise;
    const page = await this.pdf.getPage(1);
    const viewport = page.getViewport({ scale: 1 });
    const canvas = document.createElement('canvas');
    canvas.width = viewport.width;
    canvas.height = viewport.height;
    canvas.className = 'pdf-underlay';
    const ctx = canvas.getContext('2d');
    if (!ctx) {
      throw new Error('Unable to obtain canvas context');
    }
    await page.render({ canvasContext: ctx, viewport }).promise;
    this.container.appendChild(canvas);
    const [, , , pageHeight] = page.view;
    return {
      canvas,
      width: viewport.width,
      height: viewport.height,
      pageHeightPt: pageHeight,
    };
  }

  reset() {
    this.container.innerHTML = '';
    this.pdf = null;
  }
}
