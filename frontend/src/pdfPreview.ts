import { GlobalWorkerOptions, getDocument, type PDFDocumentProxy } from 'pdfjs-dist';
import workerSrc from 'pdfjs-dist/build/pdf.worker?url';

GlobalWorkerOptions.workerSrc = workerSrc;

export class PdfPreview {
  private readonly canvas: HTMLCanvasElement;
  private pdf: PDFDocumentProxy | null = null;

  constructor(canvas: HTMLCanvasElement) {
    this.canvas = canvas;
  }

  async render(data: ArrayBuffer): Promise<{ width: number; height: number }> {
    this.reset();
    this.pdf = await getDocument({ data }).promise;
    const page = await this.pdf.getPage(1);
    const viewport = page.getViewport({ scale: 1 });
    this.canvas.width = viewport.width;
    this.canvas.height = viewport.height;
    this.canvas.style.width = `${viewport.width}px`;
    this.canvas.style.height = `${viewport.height}px`;
    const context = this.canvas.getContext('2d');
    if (!context) {
      throw new Error('Failed to obtain canvas context');
    }
    context.clearRect(0, 0, viewport.width, viewport.height);
    await page.render({ canvasContext: context, viewport }).promise;
    return { width: viewport.width, height: viewport.height };
  }

  reset(): void {
    const context = this.canvas.getContext('2d');
    if (context) {
      context.clearRect(0, 0, this.canvas.width, this.canvas.height);
    }
    this.canvas.width = 0;
    this.canvas.height = 0;
    this.canvas.style.width = '0px';
    this.canvas.style.height = '0px';
    this.pdf = null;
  }
}
