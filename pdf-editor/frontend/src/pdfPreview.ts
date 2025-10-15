import type { IrPage } from "./types";

export interface PreviewPage {
  canvas: HTMLCanvasElement;
  viewport: {
    width: number;
    height: number;
    scale: number;
  };
}

export interface PdfPreviewContext {
  pages: PreviewPage[];
}

export async function initializePdfPreview(pages: IrPage[], hostId: string): Promise<PdfPreviewContext> {
  const host = document.getElementById(hostId);
  if (!host) {
    throw new Error(`Missing preview host #${hostId}`);
  }
  host.innerHTML = "";

  // Placeholder implementation. Integrate pdf.js rendering in follow-up work.
  const previewPages: PreviewPage[] = pages.map((page) => {
    const canvas = document.createElement("canvas");
    canvas.width = page.widthPt;
    canvas.height = page.heightPt;
    canvas.className = "pdf-page";
    host.appendChild(canvas);
    const ctx = canvas.getContext("2d");
    if (ctx) {
      ctx.fillStyle = "#f5f5f5";
      ctx.fillRect(0, 0, canvas.width, canvas.height);
      ctx.fillStyle = "#333";
      ctx.font = "16px sans-serif";
      ctx.fillText(`Page ${page.index + 1}`, 16, 24);
    }
    return {
      canvas,
      viewport: {
        width: canvas.width,
        height: canvas.height,
        scale: 1,
      },
    };
  });

  return { pages: previewPages };
}
