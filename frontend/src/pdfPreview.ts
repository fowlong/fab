import { GlobalWorkerOptions, getDocument, type PDFDocumentProxy } from 'pdfjs-dist';
import workerSrc from 'pdfjs-dist/build/pdf.worker?url';

GlobalWorkerOptions.workerSrc = workerSrc;

export type RenderResult = {
  canvas: HTMLCanvasElement;
  widthPx: number;
  heightPx: number;
  widthPt: number;
  heightPt: number;
};

export class PdfPreview {
  private container: HTMLElement;
  private pdf: PDFDocumentProxy | null = null;
  private canvas: HTMLCanvasElement | null = null;
  private renderInfo: RenderResult | null = null;

  constructor(container: HTMLElement) {
    this.container = container;
  }

  async render(data: ArrayBuffer): Promise<RenderResult> {
    this.reset();
    this.pdf = await getDocument({ data }).promise;
    const page = await this.pdf.getPage(1);
    const viewport = page.getViewport({ scale: 1 });
    const canvas = document.createElement('canvas');
    canvas.width = viewport.width;
    canvas.height = viewport.height;
    canvas.className = 'pdf-underlay';
    const context = canvas.getContext('2d');
    if (!context) {
      throw new Error('Failed to obtain canvas context for PDF render');
    }
    await page.render({ canvasContext: context, viewport }).promise;
    this.container.appendChild(canvas);

    const [x0, y0, x1, y1] = page.view;
    const result: RenderResult = {
      canvas,
      widthPx: canvas.width,
      heightPx: canvas.height,
      widthPt: x1 - x0,
      heightPt: y1 - y0,
    };
    this.canvas = canvas;
    this.renderInfo = result;
    return result;
  }

  getRenderInfo(): RenderResult | null {
    return this.renderInfo;
  }

  reset(): void {
    if (this.canvas) {
      this.canvas.remove();
    }
    this.container.innerHTML = '';
    this.canvas = null;
    this.renderInfo = null;
    this.pdf = null;
  }
}
