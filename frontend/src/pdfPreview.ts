import { GlobalWorkerOptions, getDocument, type PDFDocumentProxy } from 'pdfjs-dist';
import workerSrc from 'pdfjs-dist/build/pdf.worker?url';

GlobalWorkerOptions.workerSrc = workerSrc;

export type PageRender = {
  canvas: HTMLCanvasElement;
  widthPx: number;
  heightPx: number;
  widthPt: number;
  heightPt: number;
};

export class PdfPreview {
  private container: HTMLElement;
  private pdf: PDFDocumentProxy | null = null;

  constructor(container: HTMLElement) {
    this.container = container;
  }

  async renderFirstPage(data: ArrayBuffer): Promise<PageRender> {
    this.reset();
    this.pdf = await getDocument({ data }).promise;
    const page = await this.pdf.getPage(1);
    const viewport = page.getViewport({ scale: 1 });
    const canvas = document.createElement('canvas');
    canvas.width = Math.ceil(viewport.width);
    canvas.height = Math.ceil(viewport.height);
    canvas.className = 'pdf-underlay';
    const context = canvas.getContext('2d');
    if (!context) {
      throw new Error('Failed to obtain canvas context');
    }
    await page.render({ canvasContext: context, viewport }).promise;
    canvas.style.width = `${canvas.width}px`;
    canvas.style.height = `${canvas.height}px`;
    this.container.innerHTML = '';
    this.container.appendChild(canvas);
    const [x0, y0, x1, y1] = page.view;
    return {
      canvas,
      widthPx: canvas.width,
      heightPx: canvas.height,
      widthPt: x1 - x0,
      heightPt: y1 - y0,
    };
  }

  reset() {
    this.container.innerHTML = '';
    this.pdf = null;
  }
}
