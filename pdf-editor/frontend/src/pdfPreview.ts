import { getDocument, GlobalWorkerOptions, type PDFDocumentProxy } from 'pdfjs-dist';
import type { PageIR } from './types';
import { downloadPdf } from './api';

GlobalWorkerOptions.workerSrc = `https://cdnjs.cloudflare.com/ajax/libs/pdf.js/${(pdfjsLibVersion())}/pdf.worker.min.js`;

function pdfjsLibVersion(): string {
  return '4.0.269';
}

export interface PdfPreviewHandle {
  pdf: PDFDocumentProxy;
  canvases: HTMLCanvasElement[];
}

export async function initPdfPreview(pages: PageIR[], container: HTMLElement, docId?: string): Promise<PdfPreviewHandle> {
  const pdfData = await loadPdfData(docId);
  const pdf = await getDocument({ data: pdfData }).promise;

  const canvases: HTMLCanvasElement[] = [];
  for (const pageInfo of pages) {
    const page = await pdf.getPage(pageInfo.index + 1);
    const viewport = page.getViewport({ scale: 1 });
    const wrapper = document.createElement('div');
    wrapper.className = 'page-wrapper';
    wrapper.style.position = 'relative';
    const canvas = document.createElement('canvas');
    canvas.width = viewport.width;
    canvas.height = viewport.height;
    const ctx = canvas.getContext('2d');
    if (!ctx) {
      throw new Error('Failed to get canvas context');
    }
    await page.render({ canvasContext: ctx, viewport }).promise;
    canvas.className = 'pdf-page';
    canvases.push(canvas);
    wrapper.appendChild(canvas);
    container.appendChild(wrapper);
  }

  return { pdf, canvases };
}

async function loadPdfData(docId?: string): Promise<ArrayBuffer> {
  if (docId) {
    try {
      const blob = await downloadPdf(docId);
      return await blob.arrayBuffer();
    } catch (error) {
      console.warn('Falling back to bundled sample PDF', error);
    }
  }
  const res = await fetch('/sample.pdf');
  if (!res.ok) {
    throw new Error('Failed to load sample PDF');
  }
  return await res.arrayBuffer();
}
