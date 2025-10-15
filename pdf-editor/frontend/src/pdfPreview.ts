import * as pdfjsLib from "pdfjs-dist";
import workerSrc from "pdfjs-dist/build/pdf.worker.min.js?url";
import "pdfjs-dist/web/pdf_viewer.css";

pdfjsLib.GlobalWorkerOptions.workerSrc = workerSrc;

export async function renderPdf(
  container: HTMLElement,
  pdfData: ArrayBuffer,
  onPageRendered: (pageIndex: number, canvas: HTMLCanvasElement) => void
): Promise<void> {
  const pdf = await pdfjsLib.getDocument({ data: pdfData }).promise;
  container.innerHTML = "";
  for (let i = 1; i <= pdf.numPages; i += 1) {
    const page = await pdf.getPage(i);
    const viewport = page.getViewport({ scale: 1.0 });
    const canvas = document.createElement("canvas");
    const context = canvas.getContext("2d");
    if (!context) {
      throw new Error("Unable to get canvas context");
    }
    canvas.width = viewport.width;
    canvas.height = viewport.height;
    canvas.classList.add("pdf-page");
    await page.render({ canvasContext: context, viewport }).promise;
    canvas.dataset.pageIndex = String(i - 1);
    container.appendChild(canvas);
    onPageRendered(i - 1, canvas);
  }
}
