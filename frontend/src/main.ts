import "./styles.css";
import { renderPdfPreview, resetPdfPreview } from "./pdfPreview";
import { initFabricOverlay } from "./fabricOverlay";
import { loadDocumentIR, patchDocument, downloadPdf } from "./api";
import type { DocumentIR, PatchOperation } from "./types";

const app = document.getElementById("app");
if (!app) {
  throw new Error("#app container missing");
}

app.innerHTML = `
  <main class="layout">
    <aside class="sidebar" id="sidebar"></aside>
    <section class="editor">
      <header class="toolbar">
        <button data-action="move">Move / Scale</button>
        <button data-action="edit-text">Edit Text</button>
        <button data-action="style">Colour</button>
        <button data-action="opacity">Opacity</button>
        <button data-action="download">Download PDF</button>
        <span class="status" id="status"></span>
      </header>
      <section class="page-container" id="page-container"></section>
    </section>
  </main>
`;

const statusEl = document.getElementById("status");
const pageContainer = document.getElementById("page-container");

if (!pageContainer) {
  throw new Error("page container missing");
}

let currentDocId: string | null = null;

async function openFile(file: File) {
  setStatus(`Opening ${file.name}…`);
  const { docId, ir } = await loadDocumentIR(file);
  currentDocId = docId;
  setStatus("Rendering preview…");
  resetPdfPreview();
  await renderPages(ir);
  setStatus("Ready");
}

async function renderPages(ir: DocumentIR) {
  if (!pageContainer) return;
  pageContainer.innerHTML = "";

  for (const page of ir.pages) {
    const { canvas, viewport } = await renderPdfPreview(page, ir.documentMeta);
    const overlay = initFabricOverlay(canvas, page, viewport, {
      onTransform: handleTransform,
      onEditText: handleEditText,
    });

    const wrapper = document.createElement("div");
    wrapper.className = "page-wrapper";
    wrapper.append(canvas, overlay.element);
    pageContainer.append(wrapper);
  }
}

async function handleTransform(ops: PatchOperation[]) {
  if (!currentDocId) return;
  await sendPatch(ops);
}

async function handleEditText(ops: PatchOperation[]) {
  if (!currentDocId) return;
  await sendPatch(ops);
}

async function sendPatch(ops: PatchOperation[]) {
  if (!currentDocId) return;
  try {
    setStatus("Saving…");
    const response = await patchDocument(currentDocId, ops);
    setStatus(response.ok ? "Saved" : "Error saving");
  } catch (error) {
    console.error(error);
    setStatus("Failed to apply patch");
  }
}

function setStatus(message: string) {
  if (statusEl) {
    statusEl.textContent = message;
  }
}

function setupToolbar() {
  const downloadBtn = app.querySelector('[data-action="download"]');
  downloadBtn?.addEventListener("click", async () => {
    if (!currentDocId) return;
    const blob = await downloadPdf(currentDocId);
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = "edited.pdf";
    a.click();
    URL.revokeObjectURL(url);
  });
}

setupToolbar();

function setupFileInput() {
  const input = document.createElement("input");
  input.type = "file";
  input.accept = "application/pdf";
  input.className = "file-input";
  input.addEventListener("change", () => {
    const file = input.files?.[0];
    if (file) {
      void openFile(file);
    }
  });

  const sidebar = document.getElementById("sidebar");
  if (sidebar) {
    sidebar.append(input);
  }
}

setupFileInput();
