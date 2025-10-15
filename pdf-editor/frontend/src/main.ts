import "./styles.css";
import { initPdfPreview, loadPdfDocument } from "./pdfPreview";
import { initFabricOverlay } from "./fabricOverlay";
import { ApiClient } from "./api";
import { EditorMapping } from "./mapping";
import type { DocumentIR } from "./types";

const app = document.querySelector<HTMLDivElement>("#app");
if (!app) {
  throw new Error("Missing #app container");
}

const apiBase = import.meta.env.VITE_API_BASE ?? "http://localhost:8787";
const api = new ApiClient(apiBase);
const mapping = new EditorMapping();

const fileInput = document.createElement("input");
fileInput.type = "file";
fileInput.accept = "application/pdf";
fileInput.addEventListener("change", async () => {
  if (!fileInput.files?.length) return;
  const file = fileInput.files[0];
  const docId = await api.openDocument(file);
  const ir = await api.fetchIR(docId);
  renderEditor(ir, docId);
});

const toolbar = document.createElement("div");
toolbar.className = "toolbar";
toolbar.appendChild(fileInput);

const editorContainer = document.createElement("div");
editorContainer.className = "editor";

app.append(toolbar, editorContainer);

async function renderEditor(ir: DocumentIR, docId: string) {
  editorContainer.innerHTML = "";
  const previewRoot = document.createElement("div");
  previewRoot.className = "preview-root";

  const pdfUrl = ir.pdfUrl ?? api.resolvePdfUrl(docId);
  const pdf = await loadPdfDocument(pdfUrl);

  for (const page of ir.pages) {
    const { canvas: previewCanvas } = await initPdfPreview({
      page,
      container: previewRoot,
      pdf
    });
    const overlayCanvas = document.createElement("canvas");
    overlayCanvas.id = `fabric-p${page.index}`;
    overlayCanvas.width = previewCanvas.width;
    overlayCanvas.height = previewCanvas.height;
    overlayCanvas.className = "overlay";

    const stack = document.createElement("div");
    stack.className = "page-stack";
    stack.append(previewCanvas, overlayCanvas);
    previewRoot.appendChild(stack);

    await initFabricOverlay({
      canvasElement: overlayCanvas,
      page,
      docId,
      api,
      mapping
    });
  }

  editorContainer.appendChild(previewRoot);
}
