import { getDocument, GlobalWorkerOptions, type PDFDocumentProxy } from 'pdfjs-dist';

GlobalWorkerOptions.workerSrc = 'https://cdnjs.cloudflare.com/ajax/libs/pdf.js/4.0.379/pdf.worker.min.js';

export interface PdfPreviewOptions {
  container: HTMLElement;
  data: Uint8Array;
}

export async function renderPdfPreview(options: PdfPreviewOptions): Promise<PDFDocumentProxy> {
  const loadingTask = getDocument({ data: options.data });
  const pdf = await loadingTask.promise;
  const pageCount = pdf.numPages;
  options.container.innerHTML = '';

  for (let pageIndex = 1; pageIndex <= pageCount; pageIndex += 1) {
    const page = await pdf.getPage(pageIndex);
    const viewport = page.getViewport({ scale: 1.5 });

    const canvas = document.createElement('canvas');
    canvas.width = viewport.width;
    canvas.height = viewport.height;
    canvas.style.width = `${viewport.width}px`;
    canvas.style.height = `${viewport.height}px`;
    canvas.className = 'pdf-preview-canvas';

    const context = canvas.getContext('2d');
    if (!context) {
      throw new Error('Unable to acquire 2D context for pdf.js preview');
    }

    await page.render({ canvasContext: context, viewport }).promise;

    const wrapper = document.createElement('div');
    wrapper.className = 'pdf-page-wrapper';
    wrapper.appendChild(canvas);

    options.container.appendChild(wrapper);
  }

  return pdf;
}
