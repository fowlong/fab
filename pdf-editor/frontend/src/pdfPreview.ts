import { GlobalWorkerOptions } from 'pdfjs-dist';
import type { PageIR } from './types';

GlobalWorkerOptions.workerSrc = 'https://cdnjs.cloudflare.com/ajax/libs/pdf.js/4.0.379/pdf.worker.min.js';

export interface PagePreview {
  canvas: HTMLCanvasElement;
  pageIndex: number;
}

export async function createPdfPreview(pages: PageIR[], container: HTMLElement): Promise<PagePreview[]> {
  // Placeholder implementation: real rendering will use pdf.js with PDF bytes from the backend.
  return pages.map((page) => {
    const canvas = document.createElement('canvas');
    canvas.width = page.widthPt;
    canvas.height = page.heightPt;
    canvas.className = 'pdf-preview placeholder';
    const ctx = canvas.getContext('2d');
    if (ctx) {
      ctx.fillStyle = '#e2e8f0';
      ctx.fillRect(0, 0, canvas.width, canvas.height);
      ctx.fillStyle = '#475569';
      ctx.font = '16px sans-serif';
      ctx.fillText(`Page ${page.index + 1}`, 24, 36);
    }
    container.appendChild(canvas);
    return { canvas, pageIndex: page.index };
  });
}
