import { GlobalWorkerOptions, getDocument, type PDFDocumentProxy } from 'pdfjs-dist';
import type { IrDocument, PdfPreview } from './types';

const PDFJS_VERSION = '3.11.174';
GlobalWorkerOptions.workerSrc = `https://cdnjs.cloudflare.com/ajax/libs/pdf.js/${PDFJS_VERSION}/pdf.worker.min.js`;

export async function initPdfPreview(
  docId: string,
  ir: IrDocument,
  editor: HTMLElement
): Promise<PdfPreview> {
  if (!ir.pages.length) {
    throw new Error('IR has no pages');
  }

  const pdfUrl = `${(window as any).__API_BASE__ ?? 'http://localhost:8787'}/api/pdf/${docId}`;
  const pdfBytes = await fetch(pdfUrl).then((res) => res.arrayBuffer());
  const doc = await loadPdfFromBytes(pdfBytes);

  const pages = await Promise.all(
    ir.pages.map(async (pageMeta) => {
      const page = await doc.getPage(pageMeta.index + 1);
      const scale = 1.0;
      const viewport = page.getViewport({ scale });

      const container = document.createElement('div');
      container.className = 'page-container';

      const canvas = document.createElement('canvas');
      const context = canvas.getContext('2d');
      if (!context) {
        throw new Error('Failed to acquire canvas context');
      }

      canvas.height = viewport.height;
      canvas.width = viewport.width;
      canvas.dataset.scale = String(scale);
      canvas.style.width = `${viewport.width}px`;
      canvas.style.height = `${viewport.height}px`;

      container.appendChild(canvas);
      editor.appendChild(container);

      await page.render({ canvasContext: context, viewport }).promise;

      return { pageIndex: pageMeta.index, canvas, container };
    })
  );

  return { pages };
}

async function loadPdfFromBytes(data: ArrayBuffer): Promise<PDFDocumentProxy> {
  const loadingTask = getDocument({ data });
  return loadingTask.promise;
}
