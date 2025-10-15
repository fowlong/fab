import type { DocumentIR } from "./types";
import type { ApiClient } from "./api";

export interface PdfPreviewOptions {
  canvas: HTMLCanvasElement;
  api: ApiClient;
  onLoaded: (doc: DocumentIR) => void;
}

export async function initialisePdfPreview({ canvas, api, onLoaded }: PdfPreviewOptions): Promise<void> {
  const doc = await api.openLatest();
  onLoaded(doc);
  const page = doc.pages[0];
  if (!page) {
    return;
  }
  const context = canvas.getContext("2d");
  if (!context) {
    throw new Error("Failed to acquire 2D context");
  }
  const widthPx = Math.round((page.widthPt / 72) * 96);
  const heightPx = Math.round((page.heightPt / 72) * 96);
  canvas.width = widthPx;
  canvas.height = heightPx;
  context.fillStyle = "#f1f5f9";
  context.fillRect(0, 0, widthPx, heightPx);
  context.fillStyle = "#475569";
  context.font = "16px sans-serif";
  context.fillText("PDF preview placeholder", 24, 40);
}
