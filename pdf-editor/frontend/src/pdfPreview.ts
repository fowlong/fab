import { getDocument, GlobalWorkerOptions, type PDFDocumentProxy } from "pdfjs-dist";
import type { DocumentIR } from "./types";

const workerSrc = new URL("pdfjs-dist/build/pdf.worker.min.mjs", import.meta.url);
GlobalWorkerOptions.workerSrc = workerSrc.toString();

export async function initialisePdfPreview(container: HTMLElement, ir: DocumentIR) {
  if (!ir.pages.length) {
    container.textContent = "No pages in document";
    return;
  }

  const data = ir.meta?.pdfData;
  if (!data) {
    container.textContent = "PDF data unavailable";
    return;
  }

  const pdf = await loadPdfDocument(data);

  for (const pageInfo of ir.pages) {
    const page = await pdf.getPage(pageInfo.index + 1);
    const viewport = page.getViewport({ scale: 1 });
    const canvas = document.createElement("canvas");
    canvas.width = viewport.width;
    canvas.height = viewport.height;
    canvas.dataset.pageIndex = String(pageInfo.index);
    const context = canvas.getContext("2d");
    if (!context) {
      throw new Error("Failed to acquire canvas context");
    }
    await page.render({ canvasContext: context, viewport }).promise;

    const pageWrapper = document.createElement("div");
    pageWrapper.className = "page";
    pageWrapper.style.width = `${viewport.width}px`;
    pageWrapper.style.height = `${viewport.height}px`;
    pageWrapper.append(canvas);
    container.append(pageWrapper);
  }
}

async function loadPdfDocument(dataUrl: string): Promise<PDFDocumentProxy> {
  const binary = await fetch(dataUrl).then((res) => res.arrayBuffer());
  return getDocument({ data: binary }).promise;
}
