import type { IRPage } from "./types";

export interface PdfPreviewHandle {
  container: HTMLElement;
  canvases: HTMLCanvasElement[];
}

export async function initPdfPreview(
  host: HTMLElement,
  pages: IRPage[],
): Promise<PdfPreviewHandle> {
  host.innerHTML = "";

  // Placeholder canvases; pdf.js integration will render real content later.
  const canvases = pages.map((page) => {
    const wrapper = document.createElement("div");
    wrapper.className = "page-preview";
    const canvas = document.createElement("canvas");
    canvas.width = Math.round(page.widthPt);
    canvas.height = Math.round(page.heightPt);
    canvas.className = "pdf-canvas";
    wrapper.appendChild(canvas);
    host.appendChild(wrapper);
    return canvas;
  });

  return {
    container: host,
    canvases,
  };
}
