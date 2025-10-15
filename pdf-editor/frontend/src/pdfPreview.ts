import type { DocumentIR } from "./types";
import { getDocument, GlobalWorkerOptions } from "pdfjs-dist";

GlobalWorkerOptions.workerSrc = new URL("pdf.worker.min.js", import.meta.url).toString();

export function setupPdfPreview(container: HTMLElement) {
  async function renderDocument(ir: DocumentIR) {
    container.innerHTML = "";

    if (!ir.pdfDataUrl) {
      return;
    }

    const loadingTask = getDocument(ir.pdfDataUrl);
    const pdf = await loadingTask.promise;

    for (let i = 1; i <= pdf.numPages; i++) {
      const page = await pdf.getPage(i);
      const viewport = page.getViewport({ scale: 1 });
      const canvas = document.createElement("canvas");
      canvas.classList.add("pdf-canvas");
      canvas.width = viewport.width;
      canvas.height = viewport.height;
      const context = canvas.getContext("2d");
      if (!context) {
        throw new Error("Unable to acquire canvas context");
      }
      await page.render({ canvasContext: context, viewport }).promise;
      container.appendChild(canvas);
    }
  }

  return { renderDocument };
}
