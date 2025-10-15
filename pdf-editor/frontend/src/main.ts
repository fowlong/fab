import "./styles.css";
import { renderPdfPreview } from "./pdfPreview";
import { initOverlay } from "./fabricOverlay";
import { downloadPdf, fetchIR, uploadPdf } from "./api";
import type { DocumentIR, PageIR } from "./types";

const app = document.querySelector<HTMLDivElement>("#app");
if (!app) {
  throw new Error("Missing #app container");
}

app.innerHTML = `
  <div class="layout">
    <aside class="sidebar">
      <h1>PDF Editor</h1>
      <input id="file-input" type="file" accept="application/pdf" />
      <button id="download-btn" disabled>Download PDF</button>
      <div id="status"></div>
      <ul id="page-list"></ul>
    </aside>
    <main class="editor">
      <div id="preview-container"></div>
      <div id="overlay-container"></div>
    </main>
  </div>
`;

const fileInput = app.querySelector<HTMLInputElement>("#file-input");
const downloadBtn = app.querySelector<HTMLButtonElement>("#download-btn");
const statusDiv = app.querySelector<HTMLDivElement>("#status");
const pageList = app.querySelector<HTMLUListElement>("#page-list");
const previewContainer = app.querySelector<HTMLDivElement>("#preview-container");
const overlayContainer = app.querySelector<HTMLDivElement>("#overlay-container");

if (!fileInput || !downloadBtn || !statusDiv || !pageList || !previewContainer || !overlayContainer) {
  throw new Error("Missing DOM nodes");
}

let currentDocId: string | null = null;
let currentIR: DocumentIR | null = null;
let pdfData: ArrayBuffer | null = null;

fileInput.addEventListener("change", async () => {
  const file = fileInput.files?.[0];
  if (!file) return;
  setStatus("Uploading PDF...");
  try {
    const openRes = await uploadPdf(file);
    currentDocId = openRes.docId;
    pdfData = await file.arrayBuffer();
    currentIR = await fetchIR(openRes.docId);
    renderIR(currentIR);
    downloadBtn.disabled = false;
    setStatus(`Loaded document ${openRes.docId}`);
  } catch (err) {
    console.error(err);
    setStatus(`Error: ${(err as Error).message}`);
  }
});

downloadBtn.addEventListener("click", async () => {
  if (!currentDocId) return;
  setStatus("Downloading updated PDF...");
  try {
    const blob = await downloadPdf(currentDocId);
    const url = URL.createObjectURL(blob);
    const link = document.createElement("a");
    link.href = url;
    link.download = "edited.pdf";
    link.click();
    URL.revokeObjectURL(url);
    setStatus("Downloaded PDF");
  } catch (err) {
    console.error(err);
    setStatus(`Download failed: ${(err as Error).message}`);
  }
});

function setStatus(text: string) {
  statusDiv.textContent = text;
}

async function renderIR(ir: DocumentIR) {
  if (!pdfData || !currentDocId) return;
  overlayContainer.innerHTML = "";
  pageList.innerHTML = "";
  const preview = await renderPdfPreview(previewContainer, pdfData, ir);
  ir.pages.forEach((page) => {
    appendPageListItem(page);
    const pageWrapper = document.createElement("div");
    pageWrapper.classList.add("overlay-wrapper");
    overlayContainer.appendChild(pageWrapper);
    initOverlay(pageWrapper, page, {
      docId: currentDocId!,
      onPatch: (ops) => {
        console.debug("Patch submitted", ops);
      },
    });
  });
}

function appendPageListItem(page: PageIR) {
  const li = document.createElement("li");
  li.textContent = `Page ${page.index + 1}`;
  pageList.appendChild(li);
}
