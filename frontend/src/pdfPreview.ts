import { GlobalWorkerOptions, getDocument, type PDFDocumentProxy } from 'pdfjs-dist';
import workerSrc from 'pdfjs-dist/build/pdf.worker?url';

GlobalWorkerOptions.workerSrc = workerSrc;

type PageSize = { width: number; height: number };

export class PdfPreview {
  private container: HTMLElement;
  private pdf: PDFDocumentProxy | null = null;
  private pageSizes: PageSize[] = [];
  private canvases: HTMLCanvasElement[] = [];

  constructor(container: HTMLElement) {
    this.container = container;
  }

  async load(arrayBuffer: ArrayBuffer): Promise<void> {
    this.reset();
    this.pdf = await getDocument({ data: arrayBuffer }).promise;
    this.pageSizes = [];
    this.canvases = [];

    for (let pageIndex = 1; pageIndex <= this.pdf.numPages; pageIndex += 1) {
      const page = await this.pdf.getPage(pageIndex);
      const viewport = page.getViewport({ scale: 1 });
      const canvas = document.createElement('canvas');
      canvas.width = viewport.width;
      canvas.height = viewport.height;
      canvas.className = 'pdf-page-canvas';
      const ctx = canvas.getContext('2d');
      if (!ctx) {
        throw new Error('Failed to get canvas context');
      }
      await page.render({ canvasContext: ctx, viewport }).promise;
      this.container.appendChild(canvas);
      this.pageSizes.push({ width: viewport.width, height: viewport.height });
      this.canvases.push(canvas);
    }
  }

  reset(): void {
    this.container.innerHTML = '';
    this.pdf = null;
    this.pageSizes = [];
    this.canvases = [];
  }

  getSizes(): PageSize[] {
    return [...this.pageSizes];
  }

  getCanvases(): HTMLCanvasElement[] {
    return [...this.canvases];
  }
}
