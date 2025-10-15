import { GlobalWorkerOptions, getDocument, type PDFDocumentProxy } from "pdfjs-dist";
import workerUrl from "pdfjs-dist/build/pdf.worker.min.mjs?url";
import type { IrPage } from "./types";

GlobalWorkerOptions.workerSrc = workerUrl;

let pdfDoc: PDFDocumentProxy | null = null;

export async function initialisePdfPreview(page: IrPage, container: HTMLElement) {
  if (!pdfDoc) {
    throw new Error("PDF document not yet loaded");
  }

  const canvas = document.createElement("canvas");
  const context = canvas.getContext("2d");
  if (!context) throw new Error("Failed to get 2D context");

  await pdfDoc.getPage(page.index + 1).then((pdfPage) => {
    const vp = pdfPage.getViewport({ scale: 1.0 });
    canvas.height = vp.height;
    canvas.width = vp.width;
    return pdfPage.render({ canvasContext: context, viewport: vp }).promise.then(
      () => vp,
    );
  });

  canvas.dataset.pageIndex = String(page.index);
  container.appendChild(canvas);
  return canvas;
}

export async function loadPdfBlob(blob: Blob) {
  const data = new Uint8Array(await blob.arrayBuffer());
  pdfDoc = await getDocument({ data }).promise;
  return pdfDoc;
}
