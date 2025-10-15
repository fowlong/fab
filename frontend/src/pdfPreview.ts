import type { DocumentIR } from "./types";

export interface PdfPreview {
  readonly canvases: HTMLCanvasElement[];
  readonly pageHeightsPt: number[];
}

export async function bootstrapPdfPreview(
  container: HTMLElement,
  ir: DocumentIR
): Promise<PdfPreview> {
  container.innerHTML = "";
  const canvases: HTMLCanvasElement[] = [];
  const pageHeightsPt: number[] = [];

  for (const page of ir.pages) {
    const canvas = document.createElement("canvas");
    canvas.className = "pdf-preview";
    canvas.dataset.pageIndex = String(page.index);
    canvases.push(canvas);
    pageHeightsPt.push(page.heightPt);
    container.appendChild(canvas);
  }

  // Rendering is deferred; pdf.js integration would go here.
  return { canvases, pageHeightsPt };
}
