import "./styles.css";

import { openDocument, fetchIr, sendPatch, downloadPdf } from "./api";
import { renderPdf, type RenderedPage } from "./pdfPreview";
import { FabricOverlayManager } from "./fabricOverlay";
import { deriveOverlayMeta } from "./mapping";
import type { DocumentIr, Matrix, PatchOperation } from "./types";

interface AppState {
  docId: string | null;
  ir: DocumentIr | null;
  pdfBytes: Uint8Array | null;
  pages: RenderedPage[];
  overlay: FabricOverlayManager | null;
}

const state: AppState = {
  docId: null,
  ir: null,
  pdfBytes: null,
  pages: [],
  overlay: null,
};

const app = document.querySelector<HTMLDivElement>("#app");
if (!app) {
  throw new Error("#app container missing");
}

app.appendChild(createToolbar());
const sidebar = createSidebar();
const canvasShell = createCanvasShell();
app.appendChild(sidebar);
app.appendChild(canvasShell);

function createToolbar(): HTMLElement {
  const toolbar = document.createElement("div");
  toolbar.className = "toolbar";

  const fileInput = document.createElement("input");
  fileInput.type = "file";
  fileInput.accept = "application/pdf";
  fileInput.addEventListener("change", async (event) => {
    const target = event.target as HTMLInputElement;
    if (!target.files || target.files.length === 0) return;
    await openFile(target.files[0]);
    target.value = "";
  });

  const downloadButton = document.createElement("button");
  downloadButton.className = "button";
  downloadButton.textContent = "Download PDF";
  downloadButton.addEventListener("click", async () => {
    if (!state.docId) return;
    try {
      const blob = await downloadPdf(state.docId);
      const url = URL.createObjectURL(blob);
      const anchor = document.createElement("a");
      anchor.href = url;
      anchor.download = "edited.pdf";
      anchor.click();
      URL.revokeObjectURL(url);
      showToast("Download started");
    } catch (error) {
      console.error(error);
      showToast("Failed to download PDF", true);
    }
  });

  toolbar.appendChild(fileInput);
  toolbar.appendChild(downloadButton);
  return toolbar;
}

function createSidebar(): HTMLElement {
  const sidebar = document.createElement("aside");
  sidebar.className = "sidebar";
  const title = document.createElement("h2");
  title.textContent = "Pages";
  sidebar.appendChild(title);
  const list = document.createElement("ul");
  list.id = "page-list";
  sidebar.appendChild(list);
  return sidebar;
}

function createCanvasShell(): HTMLElement {
  const shell = document.createElement("div");
  shell.className = "canvas-shell";
  shell.id = "canvas-shell";
  return shell;
}

async function openFile(file: File) {
  try {
    setLoading(true);
    const { docId, ir, pdfBytes } = await openDocument(file);
    state.docId = docId;
    state.ir = ir;
    state.pdfBytes = pdfBytes;
    await renderState();
    showToast("Document loaded");
  } catch (error) {
    console.error(error);
    showToast("Failed to open document", true);
  } finally {
    setLoading(false);
  }
}

async function renderState() {
  if (!state.ir || !state.pdfBytes) return;
  const shell = document.querySelector<HTMLDivElement>("#canvas-shell");
  if (!shell) return;
  state.pages = await renderPdf(shell, state.pdfBytes);
  const pageHeightsPt = new Map<number, number>();
  for (const page of state.ir.pages) {
    pageHeightsPt.set(page.index, page.heightPt);
  }
  state.overlay?.dispose();
  state.overlay = new FabricOverlayManager(
    state.pages,
    deriveOverlayMeta(state.ir),
    {
      onTransform: handleTransform,
      onEditText: handleEditText,
    },
    pageHeightsPt
  );
  renderSidebar(state.ir);
}

function renderSidebar(ir: DocumentIr) {
  const list = document.querySelector<HTMLUListElement>("#page-list");
  if (!list) return;
  list.innerHTML = "";
  for (const page of ir.pages) {
    const item = document.createElement("li");
    item.textContent = `Page ${page.index + 1}`;
    list.appendChild(item);
  }
}

async function handleTransform(
  target: { page: number; id: string; kind: string },
  delta: Matrix
) {
  if (!state.docId) return;
  const op: PatchOperation = {
    op: "transform",
    target: { page: target.page, id: target.id },
    deltaMatrixPt: delta,
    kind: target.kind as any,
  };
  await submitPatch([op]);
}

async function handleEditText(target: { page: number; id: string }, text: string) {
  if (!state.docId) return;
  const op: PatchOperation = {
    op: "editText",
    target,
    text,
    fontPref: { preferExisting: true },
  };
  await submitPatch([op]);
}

async function submitPatch(ops: PatchOperation[]) {
  if (!state.docId) return;
  try {
    const response = await sendPatch(state.docId, ops);
    if (response.updatedPdf) {
      const pdfBytes = dataUrlToBytes(response.updatedPdf);
      if (pdfBytes) {
        state.pdfBytes = pdfBytes;
        await refreshIr();
      }
    }
    showToast("Patch applied");
  } catch (error) {
    console.error(error);
    showToast("Failed to apply patch", true);
  }
}

async function refreshIr() {
  if (!state.docId) return;
  state.ir = await fetchIr(state.docId);
  await renderState();
}

let toastTimeout: number | null = null;
function showToast(message: string, isError = false) {
  let toast = document.querySelector<HTMLDivElement>(".status-toast");
  if (!toast) {
    toast = document.createElement("div");
    toast.className = "status-toast";
    document.body.appendChild(toast);
  }
  toast.textContent = message;
  toast.style.backgroundColor = isError ? "#ef4444" : "#1f2937";
  toast.style.opacity = "1";
  if (toastTimeout) window.clearTimeout(toastTimeout);
  toastTimeout = window.setTimeout(() => {
    if (toast) toast.style.opacity = "0";
  }, 2500);
}

function setLoading(loading: boolean) {
  document.body.style.cursor = loading ? "progress" : "default";
}

function dataUrlToBytes(dataUrl: string): Uint8Array | null {
  const match = dataUrl.match(/^data:application\/pdf;base64,(.+)$/);
  if (!match) return null;
  const base64 = match[1];
  const binary = atob(base64);
  const len = binary.length;
  const bytes = new Uint8Array(len);
  for (let i = 0; i < len; i += 1) {
    bytes[i] = binary.charCodeAt(i);
  }
  return bytes;
}
