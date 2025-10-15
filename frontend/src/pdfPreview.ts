import { GlobalWorkerOptions, getDocument } from 'pdfjs-dist';
import workerSrc from 'pdfjs-dist/build/pdf.worker?url';

GlobalWorkerOptions.workerSrc = workerSrc;

export class PdfPreview {
  private container: HTMLElement;
  private canvas: HTMLCanvasElement | null = null;
  private lastData: Uint8Array | null = null;

  constructor(container: HTMLElement) {
    this.container = container;
  }

  async render(arrayBuffer: ArrayBuffer): Promise<void> {
    this.reset();
    const data = new Uint8Array(arrayBuffer);
    this.lastData = data;
    const pdf = await getDocument({ data }).promise;
    const page = await pdf.getPage(1);
    const viewport = page.getViewport({ scale: 1 });
    const canvas = document.createElement('canvas');
    canvas.width = viewport.width;
    canvas.height = viewport.height;
    canvas.className = 'pdf-underlay';
    const ctx = canvas.getContext('2d');
    if (!ctx) {
      throw new Error('Unable to obtain 2D context');
    }
    await page.render({ canvasContext: ctx, viewport }).promise;
    this.container.innerHTML = '';
    this.container.appendChild(canvas);
    this.canvas = canvas;
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

  getLastData(): Uint8Array | null {
    return this.lastData;
  }

  reset(): void {
    this.container.innerHTML = '';
    this.canvas = null;
    this.lastData = null;
  }
}
