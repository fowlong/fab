import './styles.css';
import { open, getIR, fetchPdf, patch, download } from './api';
import { PdfPreview } from './pdfPreview';
import { FabricOverlay } from './fabricOverlay';
import type { DocumentIR, PatchOperation } from './types';

const app = document.querySelector<HTMLDivElement>('#app');
if (!app) {
  throw new Error('App container missing');
}

app.innerHTML = `
  <div class="layout">
    <aside class="sidebar">
      <h1>PDF Transform MVP</h1>
      <p class="sidebar__intro">Upload a PDF and manipulate text and image transforms directly on the page.</p>
      <button id="load-sample" class="button">Load sample document</button>
      <label class="button button--secondary">
        <span>Select PDF…</span>
        <input id="file-input" type="file" accept="application/pdf" hidden />
      </label>
      <button id="download" class="button button--secondary" disabled>Download current PDF</button>
      <div id="status" class="status"></div>
    </aside>
    <main class="editor">
      <div class="page-stack">
        <div class="page-wrapper">
          <div class="page-wrapper__pdf" id="pdf-layer"></div>
          <div class="page-wrapper__overlay" id="overlay-layer"></div>
        </div>
      </div>
    </main>
  </div>
`;

const statusEl = document.getElementById('status') as HTMLDivElement;
const pdfLayer = document.getElementById('pdf-layer') as HTMLDivElement;
const overlayLayer = document.getElementById('overlay-layer') as HTMLDivElement;
const fileInput = document.getElementById('file-input') as HTMLInputElement;
const loadSampleButton = document.getElementById('load-sample') as HTMLButtonElement;
const downloadButton = document.getElementById('download') as HTMLButtonElement;

const preview = new PdfPreview(pdfLayer);
const overlay = new FabricOverlay();

let currentDocId: string | null = null;
let busy = false;

function setStatus(message: string, kind: 'info' | 'error' = 'info') {
  statusEl.textContent = message;
  statusEl.dataset.state = kind;
}

async function loadDocument(file?: File) {
  if (busy) {
    return;
  }
  busy = true;
  setStatus('Loading document…');
  try {
    const { docId } = await open(file);
    currentDocId = docId;
    downloadButton.disabled = false;

    const ir = await getIR(docId);
    const pdfData = file ? await file.arrayBuffer() : await fetchPdf(docId);
    await renderView(docId, ir, pdfData);
    setStatus('Ready. Drag the overlay handles to transform objects.');
  } catch (error) {
    console.error(error);
    setStatus(`Failed to load document: ${(error as Error).message}`, 'error');
    downloadButton.disabled = true;
    currentDocId = null;
  } finally {
    busy = false;
  }
}

async function renderView(docId: string, ir: DocumentIR, pdfData: ArrayBuffer) {
  const metrics = await preview.load(pdfData);
  overlay.initialise(overlayLayer, metrics.widthPx, metrics.heightPx);
  await overlay.render(ir, { widthPx: metrics.widthPx, heightPx: metrics.heightPx, heightPt: metrics.heightPt }, async ({ id, kind, delta }) => {
    if (!currentDocId) {
      return;
    }
    setStatus('Applying transform…');
    try {
      const op: PatchOperation = {
        op: 'transform',
        target: { page: 0, id },
        deltaMatrixPt: delta,
        kind,
      };
      await patch(docId, [op]);
      const refreshedIr = await getIR(docId);
      const updatedPdf = await fetchPdf(docId);
      await renderView(docId, refreshedIr, updatedPdf);
      setStatus('Transform applied.');
    } catch (error) {
      console.error(error);
      setStatus(`Patch failed: ${(error as Error).message}`, 'error');
    }
  });
}

loadSampleButton.addEventListener('click', () => {
  void loadDocument();
});

fileInput.addEventListener('change', (event) => {
  const input = event.target as HTMLInputElement;
  const file = input.files?.[0];
  if (!file) {
    return;
  }
  void loadDocument(file);
  input.value = '';
});

downloadButton.addEventListener('click', () => {
  if (!currentDocId) {
    return;
  }
  void download(currentDocId).catch((error) => {
    console.error(error);
    setStatus(`Download failed: ${(error as Error).message}`, 'error');
  });
});

setStatus('Upload a PDF or load the sample to begin.');
