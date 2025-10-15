import { GlobalWorkerOptions, getDocument, type PDFDocumentProxy } from 'pdfjs-dist';
import workerSrc from 'pdfjs-dist/build/pdf.worker?url';

GlobalWorkerOptions.workerSrc = workerSrc;

export class PdfPreview {
  private container: HTMLElement;
  private pdf: PDFDocumentProxy | null = null;
  private canvas: HTMLCanvasElement | null = null;
  private pageWidthPt = 0;
  private pageHeightPt = 0;

  constructor(container: HTMLElement) {
    this.container = container;
  }

  reset() {
    this.container.innerHTML = '';
    this.pdf = null;
    this.canvas = null;
    this.pageWidthPt = 0;
    this.pageHeightPt = 0;
  }

  async render(arrayBuffer: ArrayBuffer): Promise<void> {
    this.reset();
    this.pdf = await getDocument({ data: arrayBuffer }).promise;
    const page = await this.pdf.getPage(1);
    const viewport = page.getViewport({ scale: 1 });
    this.pageWidthPt = viewport.width;
    this.pageHeightPt = viewport.height;

    const canvas = document.createElement('canvas');
    canvas.width = viewport.width;
    canvas.height = viewport.height;
    canvas.className = 'pdf-underlay';
    const ctx = canvas.getContext('2d');
    if (!ctx) {
      throw new Error('Failed to get canvas context');
    }
    await page.render({ canvasContext: ctx, viewport }).promise;
    this.container.appendChild(canvas);
    this.canvas = canvas;
    await this.pdf.destroy();
    this.pdf = null;
  }

  getCanvas(): HTMLCanvasElement | null {
    return this.canvas;
  }

  getSizePx(): { width: number; height: number } | null {
    if (!this.canvas) {
      return null;
    }
    return { width: this.canvas.width, height: this.canvas.height };
  }

  getSizePt(): { widthPt: number; heightPt: number } | null {
    if (!this.pageWidthPt || !this.pageHeightPt) {
      return null;
    }
    return { widthPt: this.pageWidthPt, heightPt: this.pageHeightPt };
  }
}
