import { GlobalWorkerOptions, getDocument } from 'pdfjs-dist';
import workerSrc from 'pdfjs-dist/build/pdf.worker?url';

GlobalWorkerOptions.workerSrc = workerSrc;

type PageSize = { widthPt: number; heightPt: number; widthPx: number; heightPx: number };

export class PdfPreview {
  private container: HTMLElement;
  private canvas: HTMLCanvasElement | null = null;
  private pageSize: PageSize | null = null;

  constructor(container: HTMLElement) {
    this.container = container;
  }

  async render(arrayBuffer: ArrayBuffer): Promise<void> {
    this.reset();
    const pdf = await getDocument({ data: arrayBuffer }).promise;
    const page = await pdf.getPage(1);
    const viewport = page.getViewport({ scale: 1 });

    const canvas = document.createElement('canvas');
    canvas.width = viewport.width;
    canvas.height = viewport.height;
    canvas.className = 'pdf-underlay';
    const ctx = canvas.getContext('2d');
    if (!ctx) {
      throw new Error('Unable to acquire canvas context');
    }
    await page.render({ canvasContext: ctx, viewport }).promise;

    this.container.appendChild(canvas);
    this.canvas = canvas;
    this.pageSize = {
      widthPt: page.view[2] - page.view[0],
      heightPt: page.view[3] - page.view[1],
      widthPx: viewport.width,
      heightPx: viewport.height,
    };
  }

  getCanvas(): HTMLCanvasElement | null {
    return this.canvas;
  }

  getPageSize(): PageSize | null {
    return this.pageSize;
  }

  reset(): void {
    this.container.innerHTML = '';
    this.canvas = null;
    this.pageSize = null;
  }
}
