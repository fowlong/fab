import './styles.css';
import { renderPdf } from './pdfPreview';
import { FabricOverlayManager } from './fabricOverlay';
import { openDocument, sendPatch, downloadPdf, fetchIR } from './api';
import type { DocumentIR, PatchOperation, PdfMatrix } from './types';

const app = document.querySelector<HTMLDivElement>('#app');
if (!app) {
  throw new Error('App container not found');
}

const sidebar = document.createElement('aside');
sidebar.className = 'sidebar';
sidebar.innerHTML = `
  <h1>PDF Editor</h1>
  <p>Select a PDF file to begin editing.</p>
  <input type="file" id="file-input" accept="application/pdf" />
  <section>
    <h2>Pages</h2>
    <ol id="page-list"></ol>
  </section>
`;

const main = document.createElement('main');
main.className = 'main';
main.innerHTML = `
  <div class="toolbar">
    <button id="refresh-ir" disabled>Refresh IR</button>
    <button id="download" disabled>Download PDF</button>
  </div>
  <div class="pdf-viewer" id="viewer"></div>
`;

const toast = document.createElement('div');
toast.className = 'toast';
app.append(sidebar, main, toast);

const fileInput = sidebar.querySelector<HTMLInputElement>('#file-input');
const pageList = sidebar.querySelector<HTMLOListElement>('#page-list');
const refreshButton = main.querySelector<HTMLButtonElement>('#refresh-ir');
const downloadButton = main.querySelector<HTMLButtonElement>('#download');
const viewer = main.querySelector<HTMLDivElement>('#viewer');

if (!fileInput || !pageList || !refreshButton || !viewer || !downloadButton) {
  throw new Error('UI failed to initialize');
}

let docId: string | null = null;
let currentIR: DocumentIR | null = null;
let overlay: FabricOverlayManager | null = null;
let currentPdfBytes: Uint8Array | null = null;

fileInput.addEventListener('change', async (event) => {
  const files = (event.target as HTMLInputElement).files;
  if (!files || files.length === 0) return;
  const file = files[0];
  try {
    const { docId: newDocId, ir } = await openDocument(file);
    docId = newDocId;
    currentIR = ir;
    refreshButton.disabled = false;
    downloadButton.disabled = false;
    overlay?.dispose();
    overlay = new FabricOverlayManager({
      onTransform: (target, delta) => onTransform(target.page, target.id, target.kind, delta)
    });
    const data = new Uint8Array(await file.arrayBuffer());
    currentPdfBytes = data;
    await renderCurrentPdf();
    showToast('PDF loaded');
    populatePages(ir);
  } catch (error) {
    console.error(error);
    showToast('Failed to open PDF');
  }
});

refreshButton.addEventListener('click', async () => {
  if (!docId) return;
  try {
    currentIR = await openDocumentById(docId);
    if (currentIR) {
      populatePages(currentIR);
      overlay?.syncDocument(currentIR);
      showToast('IR refreshed');
    }
  } catch (error) {
    console.error(error);
    showToast('Failed to refresh IR');
  }
});

downloadButton.addEventListener('click', async () => {
  if (!docId) return;
  try {
    const blob = await downloadPdf(docId);
    const url = URL.createObjectURL(blob);
    const anchor = document.createElement('a');
    anchor.href = url;
    anchor.download = 'edited.pdf';
    anchor.click();
    URL.revokeObjectURL(url);
  } catch (error) {
    console.error(error);
    showToast('Download failed');
  }
});

async function renderCurrentPdf() {
  if (!viewer || !currentPdfBytes || !currentIR) return;
  const pages = await renderPdf(currentPdfBytes, viewer);
  const containers = pages.map((p) => p.container);
  overlay?.mount(currentIR.pages, containers);
  overlay?.syncDocument(currentIR);
}

async function openDocumentById(id: string) {
  return await fetchIR(id);
}

async function onTransform(
  pageIndex: number,
  objectId: string,
  kind: 'text' | 'image' | 'path',
  deltaMatrix: PdfMatrix
) {
  if (!docId) return;
  const ops: PatchOperation[] = [
    {
      op: 'transform',
      target: { page: pageIndex, id: objectId },
      kind,
      deltaMatrixPt: deltaMatrix
    }
  ];
  try {
    const response = await sendPatch(docId, ops);
    if (response.ok && response.updatedPdf) {
      const base64 = response.updatedPdf.split(',')[1];
      if (base64) {
        currentPdfBytes = decodeBase64(base64);
        await renderCurrentPdf();
      }
      showToast('Transform applied');
    }
  } catch (error) {
    console.error(error);
    showToast('Failed to apply transform');
  }
}

function populatePages(ir: DocumentIR) {
  pageList.innerHTML = '';
  ir.pages.forEach((page) => {
    const item = document.createElement('li');
    item.textContent = `Page ${page.index + 1} (${Math.round(page.widthPt)}×${Math.round(
      page.heightPt
    )} pt)`;
    pageList.append(item);
  });
}

function showToast(message: string) {
  toast.textContent = message;
  toast.classList.add('visible');
  setTimeout(() => toast.classList.remove('visible'), 2000);
}

function decodeBase64(base64: string): Uint8Array {
  const binary = atob(base64);
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i++) {
    bytes[i] = binary.charCodeAt(i);
  }
  return bytes;
}
