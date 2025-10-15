import { GlobalWorkerOptions, getDocument } from 'pdfjs-dist';
import workerSrc from 'pdfjs-dist/build/pdf.worker?url';

GlobalWorkerOptions.workerSrc = workerSrc;

export type PageRenderResult = {
  canvas: HTMLCanvasElement;
  widthPx: number;
  heightPx: number;
};

export class PdfPreview {
  private container: HTMLElement;
  private canvas: HTMLCanvasElement | null = null;

  constructor(container: HTMLElement) {
    this.container = container;
  }

  async render(data: ArrayBuffer): Promise<PageRenderResult> {
    this.reset();
    const pdf = await getDocument({ data }).promise;
    const page = await pdf.getPage(1);
    const viewport = page.getViewport({ scale: 1 });
    const canvas = document.createElement('canvas');
    const context = canvas.getContext('2d');
    if (!context) {
      throw new Error('Unable to acquire canvas context');
    }
    canvas.width = viewport.width;
    canvas.height = viewport.height;
    canvas.className = 'pdf-underlay';
    await page.render({ canvasContext: context, viewport }).promise;
    this.container.appendChild(canvas);
    this.canvas = canvas;
    return {
      canvas,
      widthPx: viewport.width,
      heightPx: viewport.height,
    };
  }

  reset() {
    this.container.innerHTML = '';
    this.canvas = null;
  }

  getCanvas(): HTMLCanvasElement | null {
    return this.canvas;
  }
}
