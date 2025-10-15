import { getDocument, GlobalWorkerOptions, type PDFDocumentProxy } from 'pdfjs-dist';
import workerSrc from 'pdfjs-dist/build/pdf.worker.min.mjs?worker&url';

import type { DocumentIr } from './types';

GlobalWorkerOptions.workerSrc = workerSrc;

export class PdfPreview {
  private container: HTMLElement;

  constructor(container: HTMLElement) {
    this.container = container;
  }

  async render(pdfData: ArrayBuffer, ir: DocumentIr): Promise<void> {
    this.container.innerHTML = '';

    if (!pdfData.byteLength) {
      const message = document.createElement('p');
      message.textContent = 'No PDF loaded yet. Upload a document to begin.';
      this.container.appendChild(message);
      return;
    }

    const pdf = await getDocument({ data: pdfData }).promise;
    const pages = ir.pages.length
      ? ir.pages.map((page) => page.index)
      : Array.from({ length: pdf.numPages }, (_, idx) => idx);
    for (const pageIndex of pages) {
      await this.renderPage(pdf, pageIndex);
    }
  }

  private async renderPage(pdf: PDFDocumentProxy, index: number): Promise<void> {
    const page = await pdf.getPage(index + 1);
    const viewport = page.getViewport({ scale: 1 });
    const wrapper = document.createElement('div');
    wrapper.className = 'page-layer';
    wrapper.style.width = `${viewport.width}px`;
    wrapper.style.height = `${viewport.height}px`;
    const canvas = document.createElement('canvas');
    canvas.width = viewport.width;
    canvas.height = viewport.height;
    canvas.className = 'pdf-preview-canvas';
    const context = canvas.getContext('2d');
    if (!context) {
      throw new Error('Unable to obtain 2D context');
    }
    await page.render({ canvasContext: context, viewport }).promise;
    wrapper.appendChild(canvas);
    this.container.appendChild(wrapper);
  }
}

