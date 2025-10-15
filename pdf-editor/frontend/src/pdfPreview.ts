import * as pdfjsLib from "pdfjs-dist";
import "pdfjs-dist/build/pdf.worker.js";

pdfjsLib.GlobalWorkerOptions.workerSrc = new URL(
  "pdfjs-dist/build/pdf.worker.js",
  import.meta.url
).toString();

export interface RenderResult {
  canvas: HTMLCanvasElement;
  wrapper: HTMLDivElement;
  pageNumber: number;
}

export async function renderPdf(
  container: HTMLElement,
  data: ArrayBuffer
): Promise<RenderResult[]> {
  const loadingTask = pdfjsLib.getDocument({ data });
  const pdf = await loadingTask.promise;
  const results: RenderResult[] = [];

  for (let pageNum = 1; pageNum <= pdf.numPages; pageNum += 1) {
    const page = await pdf.getPage(pageNum);
    const viewport = page.getViewport({ scale: 1.5 });
    const wrapper = document.createElement("div");
    wrapper.className = "canvas-wrapper";
    wrapper.style.position = "relative";
    wrapper.style.width = `${viewport.width}px`;
    wrapper.style.height = `${viewport.height}px`;

    const canvas = document.createElement("canvas");
    canvas.width = viewport.width;
    canvas.height = viewport.height;
    const ctx = canvas.getContext("2d");
    if (!ctx) {
      throw new Error("Unable to get canvas context");
    }
    wrapper.appendChild(canvas);
    container.appendChild(wrapper);

    await page.render({ canvasContext: ctx, viewport }).promise;

    results.push({ canvas, wrapper, pageNumber: pageNum - 1 });
  }

  return results;
}
