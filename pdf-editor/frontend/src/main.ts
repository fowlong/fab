import { renderPdf } from "./pdfPreview";
import { FabricOverlay } from "./fabricOverlay";
import { buildFabricDescriptors } from "./mapping";
import type { DocumentIr, PatchOperation } from "./types";
import { applyPatch, openDocument, fetchIr, downloadPdf } from "./api";

const app = document.getElementById("app");
if (!app) {
  throw new Error("Missing #app element");
}

app.innerHTML = `
  <div class="sidebar">
    <input type="file" id="file-input" accept="application/pdf" />
    <button id="download-btn" disabled>Download PDF</button>
    <div id="status"></div>
  </div>
  <div class="editor">
    <div class="toolbar">
      <button data-tool="move" class="active">Move/Scale</button>
    </div>
    <div id="page-container" class="page-container"></div>
    <div id="toast" class="toast"></div>
  </div>
`;

const fileInput = document.getElementById("file-input") as HTMLInputElement;
const downloadBtn = document.getElementById("download-btn") as HTMLButtonElement;
const pageContainer = document.getElementById("page-container") as HTMLDivElement;
const toast = document.getElementById("toast") as HTMLDivElement;

let docId: string | null = null;
let ir: DocumentIr | null = null;
const overlay = new FabricOverlay({
  onTransform: async (ops) => {
    if (!docId) return;
    await submitPatch(ops);
  },
  onTextEdit: () => {
    // Text editing UI is not implemented yet.
  },
});

fileInput.addEventListener("change", async (event) => {
  const target = event.target as HTMLInputElement;
  const file = target.files?.[0];
  if (!file) return;
  try {
    const { docId: newDocId, ir: newIr } = await openDocument(file);
    docId = newDocId;
    ir = newIr;
    await renderDoc();
    downloadBtn.disabled = false;
    showToast("PDF loaded");
  } catch (err) {
    console.error(err);
    showToast("Failed to load PDF");
  }
});

downloadBtn.addEventListener("click", async () => {
  if (!docId) return;
  const blob = await downloadPdf(docId);
  const url = URL.createObjectURL(blob);
  const link = document.createElement("a");
  link.href = url;
  link.download = "edited.pdf";
  link.click();
  URL.revokeObjectURL(url);
});

async function submitPatch(ops: PatchOperation[]) {
  if (!docId) return;
  try {
    await applyPatch(docId, ops);
    showToast("Saved incremental update");
    if (ir) {
      ir = await fetchIr(docId);
      await renderDoc();
    }
  } catch (err) {
    console.error(err);
    showToast("Patch failed");
  }
}

async function renderDoc() {
  if (!ir || !docId) return;
  overlay.dispose();
  const blob = await downloadPdf(docId);
  const buffer = await blob.arrayBuffer();
  await renderPdf(pageContainer, buffer, (pageIndex, canvas) => {
    const overlayCanvas = document.createElement("canvas");
    overlayCanvas.width = canvas.width;
    overlayCanvas.height = canvas.height;
    overlayCanvas.classList.add("fabric-overlay");
    canvas.after(overlayCanvas);
    overlay.attach(pageIndex, overlayCanvas);
    const page = ir!.pages[pageIndex];
    const descriptors = buildFabricDescriptors(page);
    overlay.renderDescriptors(pageIndex, descriptors, page.heightPt);
  });
}

function showToast(message: string) {
  toast.textContent = message;
  toast.classList.add("visible");
  setTimeout(() => toast.classList.remove("visible"), 2000);
}
