import { GlobalWorkerOptions, getDocument, type PDFDocumentProxy } from 'pdfjs-dist';
import type { DocumentIR } from './types';

GlobalWorkerOptions.workerSrc = 'https://cdnjs.cloudflare.com/ajax/libs/pdf.js/4.2.67/pdf.worker.min.js';

const PT_TO_PX = 96 / 72;

export async function initialisePdfPreview(ir: DocumentIR, container: HTMLElement) {
  if (ir.sourceUrl) {
    await renderFromPdfJs(ir, container);
  } else {
    renderFallback(ir, container);
  }
}

async function renderFromPdfJs(ir: DocumentIR, container: HTMLElement) {
  const loadingTask = getDocument(ir.sourceUrl!);
  const pdf: PDFDocumentProxy = await loadingTask.promise;

  for (const pageIR of ir.pages) {
    const page = await pdf.getPage(pageIR.index + 1);
    const viewport = page.getViewport({ scale: 1 });
    const wrapper = createPageWrapper(viewport.width, viewport.height);

    const canvas = document.createElement('canvas');
    canvas.width = viewport.width;
    canvas.height = viewport.height;
    canvas.className = 'pdf-canvas';

    const context = canvas.getContext('2d');
    if (!context) throw new Error('Canvas 2D context unavailable');

    await page.render({ canvasContext: context, viewport }).promise;

    wrapper.appendChild(canvas);
    container.appendChild(wrapper);
  }
}

function renderFallback(ir: DocumentIR, container: HTMLElement) {
  ir.pages.forEach((page) => {
    const width = page.widthPt * PT_TO_PX;
    const height = page.heightPt * PT_TO_PX;
    const wrapper = createPageWrapper(width, height);
    const placeholder = document.createElement('div');
    placeholder.className = 'pdf-canvas';
    placeholder.style.width = `${width}px`;
    placeholder.style.height = `${height}px`;
    placeholder.style.background = 'repeating-linear-gradient(45deg, #f3f3f3, #f3f3f3 10px, #e2e2e2 10px, #e2e2e2 20px)';
    wrapper.appendChild(placeholder);
    container.appendChild(wrapper);
  });
}

function createPageWrapper(width: number, height: number) {
  const wrapper = document.createElement('div');
  wrapper.className = 'page-wrapper';
  wrapper.style.setProperty('--page-width', `${width}px`);
  wrapper.style.setProperty('--page-height', `${height}px`);
  wrapper.style.width = `${width}px`;
  wrapper.style.height = `${height}px`;
  return wrapper;
}
