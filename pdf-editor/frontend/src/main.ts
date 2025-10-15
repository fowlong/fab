import "./styles.css";
import { renderDocument } from "./pdfPreview";
import { initFabricOverlay } from "./fabricOverlay";
import { loadInitialDocument, registerDownloadHandler } from "./api";

const app = document.getElementById("app");
if (!app) {
  throw new Error("App container missing");
}

app.innerHTML = `
  <main class="layout">
    <aside class="sidebar">
      <h1>PDF Editor</h1>
      <input type="file" accept="application/pdf" id="file-input" />
      <button id="download-btn" disabled>Download PDF</button>
      <div id="status"></div>
    </aside>
    <section class="editor">
      <div id="page-container"></div>
    </section>
  </main>
`;

const fileInput = document.getElementById("file-input") as HTMLInputElement;
const downloadBtn = document.getElementById("download-btn") as HTMLButtonElement;
const statusEl = document.getElementById("status") as HTMLDivElement;
const pageContainer = document.getElementById("page-container") as HTMLDivElement;

async function openDocument(file: File) {
  statusEl.textContent = "Uploading…";
  try {
    const doc = await loadInitialDocument(file);
    statusEl.textContent = "Rendering preview…";
    await renderDocument(pageContainer, doc.docId, doc.ir);
    initFabricOverlay(pageContainer, doc);
    registerDownloadHandler(downloadBtn, doc.docId);
    downloadBtn.disabled = false;
    statusEl.textContent = "Ready";
  } catch (err) {
    console.error(err);
    statusEl.textContent = err instanceof Error ? err.message : String(err);
  }
}

fileInput.addEventListener("change", async (ev) => {
  const files = (ev.target as HTMLInputElement).files;
  if (!files || files.length === 0) {
    return;
  }
  await openDocument(files[0]!);
});
