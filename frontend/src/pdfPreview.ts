import {
  GlobalWorkerOptions,
  getDocument,
  type PDFDocumentProxy,
  type PDFPageProxy,
} from 'pdfjs-dist';
import workerSrc from 'pdfjs-dist/build/pdf.worker?url';

GlobalWorkerOptions.workerSrc = workerSrc;

export type PageMetrics = {
  widthPx: number;
  heightPx: number;
  widthPt: number;
  heightPt: number;
};

export class PdfPreview {
  private container: HTMLElement;
  private canvas: HTMLCanvasElement;
  private pdf: PDFDocumentProxy | null = null;
  private metrics: PageMetrics | null = null;

  constructor(container: HTMLElement) {
    this.container = container;
    this.canvas = document.createElement('canvas');
    this.canvas.className = 'pdf-underlay';
    this.container.innerHTML = '';
    this.container.appendChild(this.canvas);
  }

  async render(arrayBuffer: ArrayBuffer): Promise<PageMetrics> {
    this.reset();
    this.pdf = await getDocument({ data: arrayBuffer }).promise;
    const page = await this.pdf.getPage(1);
    this.metrics = await this.renderPage(page);
    return this.metrics;
  }

  getCanvas(): HTMLCanvasElement {
    return this.canvas;
  }

  getMetrics(): PageMetrics | null {
    return this.metrics;
  }

  private reset() {
    const ctx = this.canvas.getContext('2d');
    if (ctx) {
      ctx.clearRect(0, 0, this.canvas.width, this.canvas.height);
    }
    this.metrics = null;
  }

  private async renderPage(page: PDFPageProxy): Promise<PageMetrics> {
    const viewport = page.getViewport({ scale: 1 });
    const context = this.canvas.getContext('2d');
    if (!context) {
      throw new Error('Canvas context unavailable');
    }
    this.canvas.width = viewport.width;
    this.canvas.height = viewport.height;
    this.canvas.style.width = `${viewport.width}px`;
    this.canvas.style.height = `${viewport.height}px`;
    await page.render({ canvasContext: context, viewport }).promise;
    const [x0, y0, x1, y1] = page.view;
    return {
      widthPx: viewport.width,
      heightPx: viewport.height,
      widthPt: x1 - x0,
      heightPt: y1 - y0,
    };
  }
}
