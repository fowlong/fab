import "./styles.css";
import { initialisePdfPreview } from "./pdfPreview";
import { initialiseFabricOverlay } from "./fabricOverlay";
import { loadDocumentIR } from "./api";

const app = document.querySelector<HTMLDivElement>("#app");
if (!app) {
  throw new Error("Missing #app root element");
}

app.innerHTML = `
  <div class="layout">
    <aside class="sidebar">
      <h1>PDF Editor</h1>
      <input id="file-input" type="file" accept="application/pdf" />
      <button id="download-btn" disabled>Download PDF</button>
      <div class="status" id="status"></div>
    </aside>
    <main class="editor" id="editor"></main>
  </div>
`;

const editor = document.querySelector<HTMLDivElement>("#editor")!;
const fileInput = document.querySelector<HTMLInputElement>("#file-input")!;
const downloadBtn = document.querySelector<HTMLButtonElement>("#download-btn")!;
const statusEl = document.querySelector<HTMLDivElement>("#status")!;

let currentDocId: string | null = null;

fileInput.addEventListener("change", async (event) => {
  const target = event.target as HTMLInputElement;
  const file = target.files?.[0];
  if (!file) {
    return;
  }

  statusEl.textContent = "Loading PDF…";
  try {
    const { docId, ir } = await loadDocumentIR(file);
    currentDocId = docId;
    downloadBtn.disabled = false;
    statusEl.textContent = "Loaded";

    editor.innerHTML = "";
    await initialisePdfPreview(editor, ir);
    initialiseFabricOverlay(editor, ir, () => currentDocId);
  } catch (error) {
    console.error(error);
    statusEl.textContent = "Failed to load PDF";
  }
});

downloadBtn.addEventListener("click", async () => {
  if (!currentDocId) return;
  statusEl.textContent = "Preparing download…";
  const { downloadPdf } = await import("./api");
  try {
    const blob = await downloadPdf(currentDocId);
    const url = URL.createObjectURL(blob);
    const anchor = document.createElement("a");
    anchor.href = url;
    anchor.download = "edited.pdf";
    anchor.click();
    URL.revokeObjectURL(url);
    statusEl.textContent = "Downloaded";
  } catch (error) {
    console.error(error);
    statusEl.textContent = "Download failed";
  }
});
