import { openDocument, fetchIr, sendPatch, download } from "./api";
import { OverlayManager } from "./fabricOverlay";
import type { PatchOp } from "./types";
import { renderPdf } from "./pdfPreview";

const app = document.querySelector<HTMLDivElement>("#app");
if (!app) throw new Error("Missing #app container");

app.innerHTML = `
  <aside>
    <h2>Documents</h2>
    <input type="file" id="fileInput" accept="application/pdf" />
    <button id="downloadBtn" disabled>Download PDF</button>
    <div class="toast" id="toast"></div>
  </aside>
  <main>
    <div class="toolbar">
      <button data-mode="move" class="active">Move/Scale</button>
    </div>
    <div id="pageContainer" class="page-stack"></div>
  </main>
`;

const fileInput = document.querySelector<HTMLInputElement>("#fileInput");
const downloadBtn = document.querySelector<HTMLButtonElement>("#downloadBtn");
const pageContainer = document.querySelector<HTMLDivElement>("#pageContainer");
const toastEl = document.querySelector<HTMLDivElement>("#toast");

let currentDocId: string | null = null;
let overlayManager: OverlayManager | null = null;

function showToast(msg: string) {
  if (!toastEl) return;
  toastEl.textContent = msg;
  toastEl.classList.add("show");
  setTimeout(() => toastEl.classList.remove("show"), 2000);
}

async function handlePatch(ops: PatchOp[]) {
  if (!currentDocId) return;
  await sendPatch(currentDocId, ops);
  showToast("Patch sent (stub)");
}

async function loadPdf(file: File) {
  if (!pageContainer) return;
  pageContainer.innerHTML = "";
  const { docId } = await openDocument(file);
  currentDocId = docId;
  downloadBtn?.removeAttribute("disabled");
  const arrayBuffer = await file.arrayBuffer();
  const pdfResults = await renderPdf(pageContainer, arrayBuffer);
  const overlayCanvases = pdfResults.map(({ wrapper }) => {
    const overlay = document.createElement("canvas");
    overlay.width = wrapper.clientWidth;
    overlay.height = wrapper.clientHeight;
    overlay.style.position = "absolute";
    overlay.style.left = "0";
    overlay.style.top = "0";
    overlay.style.pointerEvents = "auto";
    wrapper.appendChild(overlay);
    return overlay;
  });
  const ir = await fetchIr(docId);
  overlayManager = new OverlayManager(handlePatch);
  overlayManager.attachToCanvases(overlayCanvases, ir);
}

fileInput?.addEventListener("change", async (event) => {
  const target = event.target as HTMLInputElement;
  if (!target.files || !target.files[0]) return;
  await loadPdf(target.files[0]);
});

downloadBtn?.addEventListener("click", async () => {
  if (!currentDocId) return;
  const blob = await download(currentDocId);
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = "document.pdf";
  document.body.appendChild(a);
  a.click();
  a.remove();
  URL.revokeObjectURL(url);
});
