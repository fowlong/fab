import { GlobalWorkerOptions, getDocument, type PDFDocumentProxy } from 'pdfjs-dist';
import workerSrc from 'pdfjs-dist/build/pdf.worker?url';

GlobalWorkerOptions.workerSrc = workerSrc;

export class PdfPreview {
  private pdf: PDFDocumentProxy | null = null;
  private canvas: HTMLCanvasElement | null = null;
  private pixelSize: { width: number; height: number } | null = null;
  private pageSizePt: { width: number; height: number } | null = null;

  async render(container: HTMLElement, data: ArrayBuffer): Promise<void> {
    this.reset(container);
    this.pdf = await getDocument({ data }).promise;
    const page = await this.pdf.getPage(1);
    const viewport = page.getViewport({ scale: 1 });

    const canvas = document.createElement('canvas');
    canvas.width = viewport.width;
    canvas.height = viewport.height;
    canvas.className = 'pdf-underlay';
    const context = canvas.getContext('2d');
    if (!context) {
      throw new Error('Failed to initialise canvas context');
    }
    await page.render({ canvasContext: context, viewport }).promise;
    container.appendChild(canvas);

    this.canvas = canvas;
    this.pixelSize = { width: canvas.width, height: canvas.height };
    this.pageSizePt = {
      width: viewport.width * (72 / 96),
      height: viewport.height * (72 / 96),
    };
  }

  getCanvas(): HTMLCanvasElement | null {
    return this.canvas;
  }

  getPixelSize(): { width: number; height: number } | null {
    return this.pixelSize;
  }

  getPageSizePt(): { width: number; height: number } | null {
    return this.pageSizePt;
  }

  private reset(container: HTMLElement) {
    container.innerHTML = '';
    this.canvas = null;
    this.pixelSize = null;
    this.pageSizePt = null;
    this.pdf = null;
  }
}
