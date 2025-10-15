import {
  GlobalWorkerOptions,
  getDocument,
  type PDFPageProxy,
} from 'pdfjs-dist';
import workerSrc from 'pdfjs-dist/build/pdf.worker.min.mjs?url';

GlobalWorkerOptions.workerSrc = workerSrc;

export async function renderPdf(
  container: HTMLElement,
  data: Uint8Array,
): Promise<HTMLCanvasElement[]> {
  container.innerHTML = '';
  const loadingTask = getDocument({ data });
  const pdf = await loadingTask.promise;
  const canvases: HTMLCanvasElement[] = [];

  for (let i = 1; i <= pdf.numPages; i += 1) {
    const page = await pdf.getPage(i);
    const viewport = page.getViewport({ scale: 1.5 });
    const canvas = document.createElement('canvas');
    const context = canvas.getContext('2d');
    if (!context) {
      throw new Error('Unable to acquire 2D context for PDF canvas');
    }
    canvas.width = viewport.width;
    canvas.height = viewport.height;
    canvas.style.width = `${viewport.width}px`;
    canvas.style.height = `${viewport.height}px`;
    canvas.className = 'pdf-canvas';
    await renderPageToCanvas(page, context, viewport);

    const wrapper = document.createElement('div');
    wrapper.className = 'pdf-page';
    wrapper.appendChild(canvas);
    container.appendChild(wrapper);

    canvases.push(canvas);
  }

  return canvases;
}

async function renderPageToCanvas(
  page: PDFPageProxy,
  context: CanvasRenderingContext2D,
  viewport: ReturnType<PDFPageProxy['getViewport']>,
) {
  const renderContext = {
    canvasContext: context,
    viewport,
  };
  await page.render(renderContext).promise;
}
