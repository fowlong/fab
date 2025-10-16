import { GlobalWorkerOptions, getDocument, type PDFDocumentProxy } from 'pdfjs-dist';
import workerSrc from 'pdfjs-dist/build/pdf.worker?url';

GlobalWorkerOptions.workerSrc = workerSrc;

export type PageSize = {
  widthPx: number;
  heightPx: number;
  widthPt: number;
  heightPt: number;
};

export class PdfPreview {
  private canvas: HTMLCanvasElement;
  private pdf: PDFDocumentProxy | null = null;
  private lastSize: PageSize | null = null;

  constructor(canvas: HTMLCanvasElement) {
    this.canvas = canvas;
  }

  async load(buffer: ArrayBuffer): Promise<PageSize> {
    this.reset();
    this.pdf = await getDocument({ data: buffer }).promise;
    const page = await this.pdf.getPage(1);
    const viewport = page.getViewport({ scale: 1 });
    const ctx = this.canvas.getContext('2d');
    if (!ctx) {
      throw new Error('Failed to get canvas context');
    }
    this.canvas.width = viewport.width;
    this.canvas.height = viewport.height;
    this.canvas.style.width = `${viewport.width}px`;
    this.canvas.style.height = `${viewport.height}px`;
    await page.render({ canvasContext: ctx, viewport }).promise;
    const [x0, y0, x1, y1] = page.view;
    const size: PageSize = {
      widthPx: viewport.width,
      heightPx: viewport.height,
      widthPt: x1 - x0,
      heightPt: y1 - y0,
    };
    this.lastSize = size;
    return size;
  }

  async reload(buffer: ArrayBuffer): Promise<PageSize> {
    if (this.pdf) {
      await this.pdf.destroy();
      this.pdf = null;
    }
    return this.load(buffer);
  }

  getSize(): PageSize | null {
    return this.lastSize;
  }

  reset() {
    const ctx = this.canvas.getContext('2d');
    if (ctx) {
      ctx.clearRect(0, 0, this.canvas.width, this.canvas.height);
    }
    this.canvas.width = 0;
    this.canvas.height = 0;
    this.canvas.style.width = '0px';
    this.canvas.style.height = '0px';
    this.lastSize = null;
  }
}
