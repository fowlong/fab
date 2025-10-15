import { getDocument, GlobalWorkerOptions, type PDFDocumentProxy, type PDFPageProxy } from 'pdfjs-dist';
import type { DocumentIR, PageIR } from './types';

export interface PdfPreviewContext {
  container: HTMLElement;
  canvases: HTMLCanvasElement[];
  pdf: PDFDocumentProxy;
  pages: PageIR[];
}

export interface PdfPreviewOptions {
  container: HTMLElement;
  ir: DocumentIR;
}

const workerSrc = new URL('pdfjs-dist/build/pdf.worker.min.mjs', import.meta.url).toString();
GlobalWorkerOptions.workerSrc = workerSrc;

export async function createPdfPreview(options: PdfPreviewOptions): Promise<PdfPreviewContext> {
  const { container, ir } = options;
  container.innerHTML = '';

  const doc = await loadPdfFromIr(ir);
  const canvases: HTMLCanvasElement[] = [];

  for (const pageIR of ir.pages) {
    const canvas = document.createElement('canvas');
    canvas.className = 'preview-canvas';
    canvases.push(canvas);
    container.appendChild(wrapCanvas(canvas, pageIR));

    const page = await doc.getPage(pageIR.index + 1);
    await renderPageToCanvas(page, canvas);
  }

  return { container, canvases, pdf: doc, pages: ir.pages };
}

async function loadPdfFromIr(ir: DocumentIR): Promise<PDFDocumentProxy> {
  if (!ir.sourcePdf) {
    throw new Error('IR response missing `sourcePdf` data URL');
  }
  const data = atob(ir.sourcePdf.split(',')[1] ?? '');
  const bytes = new Uint8Array(data.length);
  for (let i = 0; i < data.length; i += 1) {
    bytes[i] = data.charCodeAt(i);
  }
  return getDocument({ data: bytes }).promise;
}

async function renderPageToCanvas(page: PDFPageProxy, canvas: HTMLCanvasElement) {
  const viewport = page.getViewport({ scale: 1.5 });
  const context = canvas.getContext('2d');
  if (!context) {
    throw new Error('Canvas 2D context unavailable');
  }
  canvas.width = viewport.width;
  canvas.height = viewport.height;
  await page.render({ canvasContext: context, viewport }).promise;
}

function wrapCanvas(canvas: HTMLCanvasElement, page: PageIR) {
  const wrapper = document.createElement('div');
  wrapper.className = 'page-wrapper';
  wrapper.style.width = `${page.widthPt}px`;
  wrapper.style.height = `${page.heightPt}px`;
  wrapper.appendChild(canvas);

  const overlay = document.createElement('canvas');
  overlay.dataset.pageIndex = String(page.index);
  overlay.className = 'overlay-canvas';
  wrapper.appendChild(overlay);

  return wrapper;
}
