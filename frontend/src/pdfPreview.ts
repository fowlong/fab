import { getDocument, GlobalWorkerOptions, type PDFDocumentProxy } from 'pdfjs-dist';

import workerSrc from 'pdfjs-dist/build/pdf.worker.min.js?url';

GlobalWorkerOptions.workerSrc = workerSrc;

type PageMetrics = {
  widthPt: number;
  heightPt: number;
  widthPx: number;
  heightPx: number;
};

export class PdfPreview {
  private container: HTMLElement;
  private canvas: HTMLCanvasElement | null = null;
  private metrics: PageMetrics | null = null;
  private doc: PDFDocumentProxy | null = null;

  constructor(container: HTMLElement) {
    this.container = container;
  }

  async load(data: ArrayBuffer): Promise<PageMetrics> {
    this.container.innerHTML = '';
    const loadingTask = getDocument({ data: new Uint8Array(data) });
    this.doc = await loadingTask.promise;
    const page = await this.doc.getPage(1);

    const unscaled = page.getViewport({ scale: 1 });
    const scale = 1.5;
    const viewport = page.getViewport({ scale });
    const outputScale = window.devicePixelRatio || 1;

    const canvas = document.createElement('canvas');
    canvas.className = 'pdf-underlay';
    canvas.width = Math.floor(viewport.width * outputScale);
    canvas.height = Math.floor(viewport.height * outputScale);
    canvas.style.width = `${viewport.width}px`;
    canvas.style.height = `${viewport.height}px`;

    const context = canvas.getContext('2d');
    if (!context) {
      throw new Error('Unable to acquire canvas context');
    }

    await page.render({
      canvasContext: context,
      viewport: page.getViewport({ scale: scale * outputScale }),
    }).promise;

    this.container.appendChild(canvas);
    this.canvas = canvas;
    this.metrics = {
      widthPt: unscaled.width,
      heightPt: unscaled.height,
      widthPx: viewport.width,
      heightPx: viewport.height,
    };

    return this.metrics;
  }

  getMetrics(): PageMetrics | null {
    return this.metrics;
  }

  getCanvas(): HTMLCanvasElement | null {
    return this.canvas;
  }
}
