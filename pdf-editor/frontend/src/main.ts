import "./styles.css";
import { initialisePdfPreview, loadPdfBlob } from "./pdfPreview";
import { initialiseFabricOverlay } from "./fabricOverlay";
import { ApiClient } from "./api";
import type { EditorState } from "./types";

const API_BASE = import.meta.env.VITE_API_BASE ?? "http://localhost:8787";

const api = new ApiClient(API_BASE);

const state: EditorState = {
  docId: null,
  pages: [],
  overlays: new Map(),
};

function createLayout() {
  const root = document.querySelector<HTMLDivElement>("#app");
  if (!root) {
    throw new Error("Missing #app root");
  }

  root.innerHTML = `
    <div class="toolbar">
      <button data-action="open">Open PDF</button>
      <button data-action="download" disabled>Download</button>
    </div>
    <div class="sidebar" id="page-thumbs"></div>
    <div class="editor" id="editor"></div>
    <input type="file" accept="application/pdf" id="file-input" hidden />
    <div class="toast-area" id="toast-area"></div>
  `;

  const fileInput = root.querySelector<HTMLInputElement>("#file-input");
  const openBtn = root.querySelector<HTMLButtonElement>('[data-action="open"]');
  const downloadBtn = root.querySelector<HTMLButtonElement>(
    '[data-action="download"]',
  );

  if (!fileInput || !openBtn || !downloadBtn) {
    throw new Error("Failed to initialise layout controls");
  }

  openBtn.addEventListener("click", () => fileInput.click());
  fileInput.addEventListener("change", async () => {
    if (!fileInput.files?.length) return;
    const file = fileInput.files[0];
    await openDocument(file, downloadBtn);
  });

  downloadBtn.addEventListener("click", async () => {
    if (!state.docId) return;
    try {
      const blob = await api.download(state.docId);
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = "edited.pdf";
      a.click();
      URL.revokeObjectURL(url);
      showToast("Downloaded updated PDF");
    } catch (error) {
      console.error(error);
      showToast("Failed to download PDF", true);
    }
  });
}

async function openDocument(file: File, downloadBtn: HTMLButtonElement) {
  try {
    const { docId } = await api.open(file);
    state.docId = docId;
    downloadBtn.disabled = false;
    showToast(`Opened ${file.name}`);

    await loadPdfBlob(file);

    const ir = await api.fetchIR(docId);
    state.pages = ir.pages;

    const editor = document.getElementById("editor");
    if (!editor) throw new Error("Missing editor container");
    editor.innerHTML = "";
    state.overlays.clear();

    for (const page of ir.pages) {
      const stack = document.createElement("div");
      stack.className = "canvas-stack";
      const pdfCanvas = await initialisePdfPreview(page, stack);
      const overlay = initialiseFabricOverlay({
        page,
        container: stack,
        pdfCanvas,
        api,
        state,
      });
      state.overlays.set(page.index, overlay);
      editor.append(stack);
    }
  } catch (error) {
    console.error(error);
    showToast("Failed to open PDF", true);
  }
}

function showToast(message: string, isError = false) {
  const container = document.getElementById("toast-area");
  if (!container) return;
  const el = document.createElement("div");
  el.className = "toast";
  el.textContent = message;
  if (isError) {
    el.style.borderColor = "rgba(255, 99, 132, 0.6)";
  }
  container.append(el);
  setTimeout(() => el.remove(), 4000);
}

createLayout();
