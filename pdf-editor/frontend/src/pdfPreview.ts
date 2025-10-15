import { getDocument, GlobalWorkerOptions, type PDFDocumentProxy } from "pdfjs-dist";
import type { DocumentIR } from "./types";

const PDF_JS_WORKER = new URL("pdfjs-dist/build/pdf.worker.min.mjs", import.meta.url);
GlobalWorkerOptions.workerSrc = PDF_JS_WORKER.toString();

export async function renderDocument(container: HTMLElement, docId: string, ir: DocumentIR) {
  container.innerHTML = "";
  const pdfBytes = await fetchPagePdf(docId, ir);
  const pdf = await loadPdf(pdfBytes);

  for (const pageInfo of ir.pages) {
    const page = await pdf.getPage(pageInfo.index + 1);
    const viewport = page.getViewport({ scale: 1 });
    const canvas = document.createElement("canvas");
    canvas.width = viewport.width;
    canvas.height = viewport.height;
    canvas.className = "pdf-canvas";
    const ctx = canvas.getContext("2d");
    if (!ctx) {
      throw new Error("2D context unavailable");
    }
    await page.render({ canvasContext: ctx, viewport }).promise;

    const wrapper = document.createElement("div");
    wrapper.className = "page-wrapper";
    wrapper.dataset.pageIndex = String(pageInfo.index);
    wrapper.appendChild(canvas);
    const overlay = document.createElement("canvas");
    overlay.id = `fabric-p${pageInfo.index}`;
    overlay.width = canvas.width;
    overlay.height = canvas.height;
    overlay.className = "fabric-overlay";
    overlay.dataset.interactive = "true";
    wrapper.appendChild(overlay);
    container.appendChild(wrapper);
  }
}

async function fetchPagePdf(docId: string, ir: DocumentIR): Promise<ArrayBuffer> {
  if (ir.pages.length === 0) {
    throw new Error("Document has no pages");
  }
  const res = await fetch(`${__API_BASE__}/api/pdf/${encodeURIComponent(docId)}`);
  if (!res.ok) {
    throw new Error("Failed to load PDF bytes");
  }
  return await res.arrayBuffer();
}

async function loadPdf(bytes: ArrayBuffer): Promise<PDFDocumentProxy> {
  return await getDocument({ data: bytes }).promise;
}
