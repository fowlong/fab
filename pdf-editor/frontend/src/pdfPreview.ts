import { GlobalWorkerOptions, getDocument, type PDFDocumentProxy } from 'pdfjs-dist';
import workerSrc from 'pdfjs-dist/build/pdf.worker.min?url';

GlobalWorkerOptions.workerSrc = workerSrc;

export class PdfPreview {
  private root: HTMLElement;
  private canvases: HTMLCanvasElement[] = [];
  private doc: PDFDocumentProxy | null = null;

  constructor(root: HTMLElement) {
    this.root = root;
  }

  reset(): void {
    this.root.innerHTML = '';
    this.canvases = [];
    this.doc = null;
  }

  async render(data: Uint8Array): Promise<void> {
    this.reset();
    this.doc = await getDocument({ data }).promise;
    const pageCount = this.doc.numPages;
    for (let pageIndex = 1; pageIndex <= pageCount; pageIndex++) {
      const page = await this.doc.getPage(pageIndex);
      const viewport = page.getViewport({ scale: 1.0 });
      const canvas = document.createElement('canvas');
      canvas.width = viewport.width;
      canvas.height = viewport.height;
      canvas.className = 'pdf-preview';
      this.root.appendChild(canvas);
      const context = canvas.getContext('2d');
      if (!context) {
        throw new Error('Failed to acquire canvas context');
      }
      await page.render({ canvasContext: context, viewport }).promise;
      this.canvases.push(canvas);
    }
  }

  getCanvasForPage(index: number): HTMLCanvasElement | undefined {
    return this.canvases[index] ?? undefined;
  }
}
