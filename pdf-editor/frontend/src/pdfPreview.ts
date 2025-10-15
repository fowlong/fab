import { GlobalWorkerOptions, getDocument, type PDFPageProxy } from "pdfjs-dist";
import type { PageIR } from "./types";

GlobalWorkerOptions.workerSrc = "https://cdnjs.cloudflare.com/ajax/libs/pdf.js/4.2.67/pdf.worker.min.js";

export async function renderPdfPreview(container: HTMLElement, pages: PageIR[]): Promise<HTMLCanvasElement[]> {
  container.classList.add("pdf-preview-host");
  const canvases: HTMLCanvasElement[] = [];

  for (const page of pages) {
    const canvas = document.createElement("canvas");
    canvas.width = Math.round((page.widthPt / 72) * 96);
    canvas.height = Math.round((page.heightPt / 72) * 96);
    canvas.className = "pdf-underlay";
    container.appendChild(canvas);
    canvases.push(canvas);
  }

  return canvases;
}

export async function rasterisePdfIntoCanvases(pdfData: ArrayBuffer, canvases: HTMLCanvasElement[]) {
  const pdf = await getDocument({ data: pdfData }).promise;
  await Promise.all(
    canvases.map(async (canvas, index) => {
      const page = await pdf.getPage(index + 1);
      await renderPageToCanvas(page, canvas);
    }),
  );
}

async function renderPageToCanvas(page: PDFPageProxy, canvas: HTMLCanvasElement) {
  const viewport = page.getViewport({ scale: canvas.width / page.getViewport({ scale: 1 }).width });
  const context = canvas.getContext("2d");
  if (!context) return;
  canvas.height = viewport.height;
  canvas.width = viewport.width;
  await page.render({ canvasContext: context, viewport }).promise;
}
