import { GlobalWorkerOptions, getDocument, type PDFDocumentProxy } from 'pdfjs-dist';
import workerSrc from 'pdfjs-dist/build/pdf.worker?url';

GlobalWorkerOptions.workerSrc = workerSrc;

export class PdfPreview {
  private container: HTMLElement;
  private pdf: PDFDocumentProxy | null = null;
  private canvas: HTMLCanvasElement | null = null;
  private pageSizePt: { width: number; height: number } | null = null;

  constructor(container: HTMLElement) {
    this.container = container;
  }

  async load(data: ArrayBuffer) {
    this.reset();
    this.pdf = await getDocument({ data }).promise;
    if (this.pdf.numPages === 0) {
      throw new Error('PDF has no pages');
    }
    const page = await this.pdf.getPage(1);
    const viewport = page.getViewport({ scale: 1 });
    const canvas = document.createElement('canvas');
    canvas.width = viewport.width;
    canvas.height = viewport.height;
    canvas.className = 'pdf-page-canvas';
    const ctx = canvas.getContext('2d');
    if (!ctx) {
      throw new Error('Failed to get canvas context');
    }
    await page.render({ canvasContext: ctx, viewport }).promise;
    this.container.appendChild(canvas);
    this.canvas = canvas;
    const [x0, y0, x1, y1] = page.view;
    this.pageSizePt = { width: x1 - x0, height: y1 - y0 };
  }

  reset() {
    this.container.innerHTML = '';
    this.pdf = null;
    this.canvas = null;
    this.pageSizePt = null;
  }

  getCanvas(): HTMLCanvasElement | null {
    return this.canvas;
  }

  getPageSizePt(): { width: number; height: number } | null {
    return this.pageSizePt;
  }
}
