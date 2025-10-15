import "./styles.css";
import { setupPdfPreview } from "./pdfPreview";
import { setupFabricOverlay } from "./fabricOverlay";
import { ApiClient } from "./api";

async function bootstrap() {
  const app = document.querySelector<HTMLDivElement>("#app");
  if (!app) {
    throw new Error("Missing root container");
  }

  app.innerHTML = `
    <div class="layout">
      <header class="toolbar">
        <h1>PDF Editor</h1>
        <div class="toolbar-actions">
          <button id="openPdf">Open PDF</button>
          <button id="downloadPdf" disabled>Download</button>
        </div>
      </header>
      <main class="workspace">
        <aside class="sidebar">
          <h2>Pages</h2>
          <ul id="pageList"></ul>
        </aside>
        <section class="canvas-stack" id="canvasStack"></section>
      </main>
      <input type="file" id="fileInput" accept="application/pdf" hidden />
    </div>
  `;

  const api = new ApiClient();
  const preview = setupPdfPreview(document.querySelector("#canvasStack") as HTMLElement);
  const overlay = setupFabricOverlay();

  const openButton = document.querySelector<HTMLButtonElement>("#openPdf");
  const downloadButton = document.querySelector<HTMLButtonElement>("#downloadPdf");
  const fileInput = document.querySelector<HTMLInputElement>("#fileInput");

  if (!openButton || !downloadButton || !fileInput) {
    throw new Error("Missing UI controls");
  }

  openButton.addEventListener("click", () => fileInput.click());

  fileInput.addEventListener("change", async () => {
    const file = fileInput.files?.[0];
    if (!file) return;

    const { docId, ir } = await api.open(file);
    preview.renderDocument(ir);
    overlay.load(ir, api, docId);
    downloadButton.disabled = false;
  });

  downloadButton.addEventListener("click", async () => {
    const current = overlay.currentDocument();
    if (!current) return;

    const blob = await api.download(current.docId);
    const url = URL.createObjectURL(blob);
    const anchor = document.createElement("a");
    anchor.href = url;
    anchor.download = "document.pdf";
    anchor.click();
    URL.revokeObjectURL(url);
  });
}

bootstrap().catch((err) => {
  console.error("Failed to bootstrap application", err);
});
