import type { DocumentIr } from './types';

export interface PdfPreviewContext {
  container: HTMLDivElement;
  canvases: HTMLCanvasElement[];
  destroy(): void;
}

async function renderPdfFromUrl(
  container: HTMLDivElement,
  ir: DocumentIr,
  canvases: HTMLCanvasElement[],
  url: string
) {
  const pdfjs = await import('pdfjs-dist');
  const workerSrc = await import('pdfjs-dist/build/pdf.worker.mjs');
  (pdfjs.GlobalWorkerOptions as unknown as { workerSrc: string }).workerSrc =
    workerSrc.default;
  const response = await fetch(url);
  if (!response.ok) {
    throw new Error('Failed to download source PDF for preview');
  }
  const buffer = await response.arrayBuffer();
  const loadingTask = pdfjs.getDocument({ data: buffer });
  const pdf = await loadingTask.promise;

  for (const pageInfo of ir.pages) {
    const page = await pdf.getPage(pageInfo.index + 1);
    const viewport = page.getViewport({ scale: 1.0 });

    const wrapper = document.createElement('div');
    wrapper.className = 'page-wrapper';

    const canvas = document.createElement('canvas');
    canvas.width = viewport.width;
    canvas.height = viewport.height;
    canvas.className = 'pdf-canvas';

    const context = canvas.getContext('2d');
    if (context) {
      await page.render({ canvasContext: context, viewport }).promise;
    }

    wrapper.appendChild(canvas);
    container.appendChild(wrapper);
    canvases.push(canvas);
  }
}

function renderPlaceholder(
  container: HTMLDivElement,
  canvases: HTMLCanvasElement[]
) {
  const wrapper = document.createElement('div');
  wrapper.className = 'page-wrapper placeholder';
  const canvas = document.createElement('canvas');
  canvas.width = 595;
  canvas.height = 842;
  const ctx = canvas.getContext('2d');
  if (ctx) {
    ctx.fillStyle = '#f8fafc';
    ctx.fillRect(0, 0, canvas.width, canvas.height);
    ctx.fillStyle = '#64748b';
    ctx.font = '20px sans-serif';
    ctx.fillText('PDF preview pending backend implementation', 40, 100);
  }
  wrapper.appendChild(canvas);
  container.appendChild(wrapper);
  canvases.push(canvas);
}

export async function initialisePdfPreview(
  container: HTMLDivElement,
  ir: DocumentIr
): Promise<PdfPreviewContext> {
  container.innerHTML = '';
  const canvases: HTMLCanvasElement[] = [];

  try {
    if (ir.sourcePdfUrl) {
      await renderPdfFromUrl(container, ir, canvases, ir.sourcePdfUrl);
    } else {
      renderPlaceholder(container, canvases);
    }
  } catch (error) {
    console.error(error);
    renderPlaceholder(container, canvases);
  }

  return {
    container,
    canvases,
    destroy() {
      container.innerHTML = '';
    }
  };
}
