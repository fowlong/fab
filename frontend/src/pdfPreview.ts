import { GlobalWorkerOptions, getDocument } from 'pdfjs-dist';
import workerSrc from 'pdfjs-dist/build/pdf.worker?url';

GlobalWorkerOptions.workerSrc = workerSrc;

export type PageMetrics = {
  widthPx: number;
  heightPx: number;
  widthPt: number;
  heightPt: number;
  canvas: HTMLCanvasElement;
};

export class PdfPreview {
  private container: HTMLElement;
  private canvas: HTMLCanvasElement | null = null;
  private ctx: CanvasRenderingContext2D | null = null;

  constructor(container: HTMLElement) {
    this.container = container;
  }

  reset() {
    this.container.innerHTML = '';
    this.canvas = null;
    this.ctx = null;
  }

  async render(data: ArrayBuffer): Promise<PageMetrics> {
    const pdf = await getDocument({ data }).promise;
    const page = await pdf.getPage(1);
    const viewport = page.getViewport({ scale: 1 });
    const [x0, y0, x1, y1] = viewport.viewBox;
    const widthPt = x1 - x0;
    const heightPt = y1 - y0;

    if (!this.canvas) {
      this.canvas = document.createElement('canvas');
      this.canvas.className = 'pdf-underlay';
      this.container.appendChild(this.canvas);
    }
    this.canvas.width = viewport.width;
    this.canvas.height = viewport.height;
    this.canvas.style.width = `${viewport.width}px`;
    this.canvas.style.height = `${viewport.height}px`;

    this.ctx = this.canvas.getContext('2d');
    if (!this.ctx) {
      throw new Error('Unable to acquire 2D context');
    }

    this.ctx.clearRect(0, 0, this.canvas.width, this.canvas.height);
    await page.render({ canvasContext: this.ctx, viewport }).promise;

    return {
      widthPx: viewport.width,
      heightPx: viewport.height,
      widthPt,
      heightPt,
      canvas: this.canvas,
    };
  }
}
