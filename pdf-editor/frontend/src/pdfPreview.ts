import type { DocumentIR, PageIR } from "./types";
import { getDocument, GlobalWorkerOptions } from "pdfjs-dist";
import pdfWorker from "pdfjs-dist/build/pdf.worker?url";

GlobalWorkerOptions.workerSrc = pdfWorker;

export interface PdfPreview {
  container: HTMLElement;
  pageCanvases: HTMLCanvasElement[];
}

export async function renderPdfPreview(
  container: HTMLElement,
  fileData: ArrayBuffer,
  ir: DocumentIR,
): Promise<PdfPreview> {
  container.innerHTML = "";
  const loadingTask = getDocument({ data: fileData });
  const pdf = await loadingTask.promise;
  const canvases: HTMLCanvasElement[] = [];

  for (const pageIR of ir.pages) {
    const page = await pdf.getPage(pageIR.index + 1);
    const viewport = page.getViewport({ scale: 1.0 });
    const canvas = document.createElement("canvas");
    const context = canvas.getContext("2d");
    if (!context) throw new Error("Unable to get canvas context");
    canvas.width = viewport.width;
    canvas.height = viewport.height;
    canvas.classList.add("pdf-preview-canvas");
    await page.render({ canvasContext: context, viewport }).promise;
    container.appendChild(canvas);
    canvases.push(canvas);
  }

  return { container, pageCanvases: canvases };
}

export function pageToCanvas(page: PageIR, preview: PdfPreview): HTMLCanvasElement | undefined {
  return preview.pageCanvases[page.index];
}
