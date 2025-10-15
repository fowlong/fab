import { DocumentIR, PageIR } from "./types";

type PreviewHandle = {
  canvases: HTMLCanvasElement[];
};

export async function initPdfPreview(
  ir: DocumentIR,
  container: HTMLElement | null
): Promise<PreviewHandle> {
  if (!container) {
    throw new Error("Missing page container");
  }

  container.innerHTML = "";
  const canvases: HTMLCanvasElement[] = [];

  ir.pages.forEach((page: PageIR, index) => {
    const wrapper = document.createElement("div");
    wrapper.className = "page-wrapper";
    const canvas = document.createElement("canvas");
    canvas.id = `pdf-page-${index}`;
    canvas.width = Math.round((page.widthPt / 72) * 96);
    canvas.height = Math.round((page.heightPt / 72) * 96);
    wrapper.appendChild(canvas);
    container.appendChild(wrapper);
    canvases.push(canvas);
  });

  return { canvases };
}
