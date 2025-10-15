import { renderPdfPreview } from "./pdfPreview";
import { FabricOverlay } from "./fabricOverlay";
import { ApiClient } from "./api";
import { FabricToPdfMapper } from "./mapping";
import type { DocumentIR, PatchOperation } from "./types";

const dropZone = document.getElementById("editor") as HTMLDivElement;
const statusEl = document.getElementById("status") as HTMLSpanElement;
const downloadBtn = document.getElementById("download") as HTMLButtonElement;

const api = new ApiClient();
let currentDocId: string | null = null;
let currentIR: DocumentIR | null = null;
let overlay: FabricOverlay | null = null;

function setStatus(message: string) {
  statusEl.textContent = message;
}

async function openFile(file: File) {
  setStatus(`Uploading ${file.name}…`);
  const { docId } = await api.open(file);
  currentDocId = docId;

  const ir = await api.fetchIR(docId);
  currentIR = ir;
  setStatus(`Loaded ${file.name}.`);

  const previewHost = document.getElementById("editor");
  if (!previewHost) {
    throw new Error("Missing editor host");
  }

  previewHost.innerHTML = "";
  const canvases = await renderPdfPreview(previewHost, ir.pages);

  overlay?.dispose();
  overlay = new FabricOverlay(previewHost, canvases, ir, async (ops) => {
    if (!currentDocId) return;
    setStatus("Saving patch…");
    const response = await api.patch(currentDocId, ops);
    setStatus(response.ok ? "Saved" : "Patch failed");
    if (response.updatedPdf) {
      downloadBtn.dataset.blobUrl = response.updatedPdf;
    }
  });

  overlay.mount();
}

function setupDnD() {
  dropZone.addEventListener("dragover", (event) => {
    event.preventDefault();
    dropZone.classList.add("dragover");
  });
  dropZone.addEventListener("dragleave", () => {
    dropZone.classList.remove("dragover");
  });
  dropZone.addEventListener("drop", (event) => {
    event.preventDefault();
    dropZone.classList.remove("dragover");
    const file = event.dataTransfer?.files?.[0];
    if (file) {
      void openFile(file);
    }
  });
}

function setupDownload() {
  downloadBtn.addEventListener("click", () => {
    if (!downloadBtn.dataset.blobUrl) {
      return;
    }
    const link = document.createElement("a");
    link.href = downloadBtn.dataset.blobUrl;
    link.download = "edited.pdf";
    link.click();
  });
}

setupDnD();
setupDownload();

// expose for debugging
Object.assign(window, {
  api,
  applyPatch(ops: PatchOperation[]) {
    if (!currentDocId || !overlay) return;
    void overlay.applyPatch(ops);
  },
  get documentIR() {
    return currentIR;
  },
  get mapper() {
    return FabricToPdfMapper;
  },
});
