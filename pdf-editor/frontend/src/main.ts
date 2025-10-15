import './styles.css';
import { renderPdf } from './pdfPreview';
import { createOverlay, type OverlayController } from './fabricOverlay';
import { downloadPdf, fetchIR, openDocument, postPatch } from './api';
import type { DocumentIR, PageObject } from './types';
import type { Matrix } from './coords';

const app = document.getElementById('app');
if (!app) {
  throw new Error('App container not found');
}

app.innerHTML = `
  <header>
    <label class="upload-control">
      <button type="button">Upload PDF</button>
      <input id="file-input" type="file" accept="application/pdf" />
    </label>
    <button id="download-btn" type="button" disabled>Download PDF</button>
    <span id="status" aria-live="polite"></span>
  </header>
  <main>
    <aside class="sidebar">
      <h2>Pages</h2>
      <div id="thumbnails">Select a PDF to begin.</div>
    </aside>
    <section class="canvas-area">
      <div class="page-stack" id="page-stack"></div>
    </section>
  </main>
  <div class="toast-container" id="toast-container"></div>
`;

const fileInput = app.querySelector<HTMLInputElement>('#file-input');
const downloadButton = app.querySelector<HTMLButtonElement>('#download-btn');
const pageStack = app.querySelector<HTMLDivElement>('#page-stack');
const thumbnails = app.querySelector<HTMLDivElement>('#thumbnails');
const status = app.querySelector<HTMLSpanElement>('#status');
const toastContainer = app.querySelector<HTMLDivElement>('#toast-container');

if (!fileInput || !downloadButton || !pageStack || !thumbnails || !status || !toastContainer) {
  throw new Error('Failed to initialise UI elements');
}

let currentDocId: string | null = null;
let currentPdfData: Uint8Array | null = null;
let currentIR: DocumentIR | null = null;
let overlays: OverlayController[] = [];

fileInput.addEventListener('change', async (event) => {
  const input = event.target as HTMLInputElement;
  const file = input.files?.[0];
  if (!file) return;
  await openFile(file);
  input.value = '';
});

downloadButton.addEventListener('click', async () => {
  if (!currentDocId) return;
  try {
    const blob = await downloadPdf(currentDocId);
    const url = URL.createObjectURL(blob);
    const anchor = document.createElement('a');
    anchor.href = url;
    anchor.download = 'edited.pdf';
    anchor.click();
    setTimeout(() => URL.revokeObjectURL(url), 5000);
    showToast('Downloaded latest PDF');
  } catch (error) {
    console.error(error);
    showToast('Failed to download PDF', true);
  }
});

async function openFile(file: File) {
  setStatus('Uploading…');
  try {
    const pdfBytes = new Uint8Array(await file.arrayBuffer());
    const { docId } = await openDocument(file);
    currentDocId = docId;
    currentPdfData = pdfBytes;
    downloadButton.disabled = false;
    showToast('PDF uploaded');
    await loadIr();
  } catch (error) {
    console.error(error);
    showToast('Failed to upload PDF', true);
  } finally {
    clearStatus();
  }
}

async function loadIr() {
  if (!currentDocId || !currentPdfData) return;
  setStatus('Loading document…');
  try {
    currentIR = await fetchIR(currentDocId);
    await renderPages(currentIR, currentPdfData);
    renderThumbnails(currentIR);
    showToast('Document ready');
  } catch (error) {
    console.error(error);
    showToast('Failed to load document', true);
  } finally {
    clearStatus();
  }
}

async function renderPages(ir: DocumentIR, data: Uint8Array) {
  clearOverlays();
  pageStack.innerHTML = '';
  const canvases = await renderPdf(pageStack, data);
  canvases.forEach((pdfCanvas, index) => {
    const overlayCanvas = document.createElement('canvas');
    overlayCanvas.width = pdfCanvas.width;
    overlayCanvas.height = pdfCanvas.height;
    overlayCanvas.style.width = `${pdfCanvas.width}px`;
    overlayCanvas.style.height = `${pdfCanvas.height}px`;
    overlayCanvas.style.position = 'absolute';
    overlayCanvas.style.left = '0';
    overlayCanvas.style.top = '0';
    overlayCanvas.style.zIndex = '10';
    pdfCanvas.parentElement?.appendChild(overlayCanvas);
    const page = ir.pages[index];
    const controller = createOverlay(
      overlayCanvas,
      page.heightPt,
      index,
      page.objects,
      handleTransform,
    );
    overlays.push(controller);
  });
}

function renderThumbnails(ir: DocumentIR) {
  thumbnails.innerHTML = '';
  ir.pages.forEach((page, index) => {
    const div = document.createElement('div');
    div.textContent = `Page ${index + 1} – ${(page.widthPt / 72).toFixed(2)} × ${(page.heightPt / 72).toFixed(2)} in`;
    thumbnails.appendChild(div);
  });
}

function clearOverlays() {
  overlays.forEach((overlay) => overlay.dispose());
  overlays = [];
}

async function handleTransform(object: PageObject, pageIndex: number, delta: Matrix) {
  if (!currentDocId) return;
  try {
    await postPatch(currentDocId, [
      {
        op: 'transform',
        target: { page: pageIndex, id: object.id },
        deltaMatrixPt: delta,
        kind: object.kind,
      },
    ]);
    showToast(`Transform applied to ${object.id}`);
  } catch (error) {
    console.error(error);
    showToast('Failed to apply transform', true);
  }
}

function setStatus(message: string) {
  status.textContent = message;
}

function clearStatus() {
  status.textContent = '';
}

function showToast(message: string, isError = false) {
  const toast = document.createElement('div');
  toast.className = 'toast';
  toast.textContent = message;
  if (isError) {
    toast.style.background = 'rgba(220, 38, 38, 0.95)';
  }
  toastContainer.appendChild(toast);
  setTimeout(() => {
    toast.remove();
  }, 3500);
}
