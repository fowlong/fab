import { initializePdfPreview } from "./pdfPreview";
import { initializeFabricOverlay } from "./fabricOverlay";
import { createApiClient } from "./api";
import "./styles.css";

const app = document.getElementById("app");
if (!app) {
  throw new Error("Missing #app container");
}

const layout = document.createElement("div");
layout.className = "layout";
layout.innerHTML = `
  <aside class="sidebar">
    <input type="file" id="fileInput" accept="application/pdf" />
    <button id="downloadBtn" disabled>Download PDF</button>
    <div id="status"></div>
  </aside>
  <main class="editor">
    <div id="canvasHost"></div>
  </main>
`;
app.appendChild(layout);

const fileInput = layout.querySelector<HTMLInputElement>("#fileInput");
const downloadBtn = layout.querySelector<HTMLButtonElement>("#downloadBtn");
const status = layout.querySelector<HTMLDivElement>("#status");

const api = createApiClient();
let currentDocId: string | null = null;

fileInput?.addEventListener("change", async (event) => {
  const file = (event.target as HTMLInputElement).files?.[0];
  if (!file) {
    return;
  }

  status!.textContent = "Uploading PDF...";
  const docId = await api.openDocument(file);
  currentDocId = docId;
  downloadBtn!.disabled = false;

  const ir = await api.loadIr(docId);
  const preview = await initializePdfPreview(ir.pages, "canvasHost");
  initializeFabricOverlay({
    pages: ir.pages,
    preview,
    api,
    docId,
    onStatus: (message) => {
      status!.textContent = message;
    },
  });
  status!.textContent = "Ready";
});

downloadBtn?.addEventListener("click", async () => {
  if (!currentDocId) {
    return;
  }
  const blob = await api.download(currentDocId);
  const url = URL.createObjectURL(blob);
  const anchor = document.createElement("a");
  anchor.href = url;
  anchor.download = "edited.pdf";
  anchor.click();
  URL.revokeObjectURL(url);
});
