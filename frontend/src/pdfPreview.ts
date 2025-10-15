import { getDocument, GlobalWorkerOptions, type PDFDocumentProxy } from "pdfjs-dist";

GlobalWorkerOptions.workerSrc = new URL("pdf.worker.min.js", import.meta.url).toString();

export interface RenderedPage {
  pageIndex: number;
  canvas: HTMLCanvasElement;
  container: HTMLDivElement;
  width: number;
  height: number;
}

export async function renderPdf(
  container: HTMLElement,
  pdfBytes: Uint8Array
): Promise<RenderedPage[]> {
  container.innerHTML = "";
  const loadingTask = getDocument({ data: pdfBytes });
  const pdf: PDFDocumentProxy = await loadingTask.promise;
  const renderedPages: RenderedPage[] = [];

  for (let pageIndex = 0; pageIndex < pdf.numPages; pageIndex += 1) {
    const page = await pdf.getPage(pageIndex + 1);
    const viewport = page.getViewport({ scale: 1.0 });
    const canvas = document.createElement("canvas");
    const context = canvas.getContext("2d");
    if (!context) {
      throw new Error("Unable to obtain canvas context");
    }
    canvas.width = viewport.width;
    canvas.height = viewport.height;

    const containerEl = document.createElement("div");
    containerEl.className = "canvas-page";
    containerEl.style.width = `${viewport.width}px`;
    containerEl.style.height = `${viewport.height}px`;
    containerEl.appendChild(canvas);

    const renderTask = page.render({ canvasContext: context, viewport });
    await renderTask.promise;

    container.appendChild(containerEl);
    renderedPages.push({
      pageIndex,
      canvas,
      container: containerEl,
      width: viewport.width,
      height: viewport.height,
    });
  }

  return renderedPages;
}
