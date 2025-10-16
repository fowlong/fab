import './styles.css';
import { PdfPreview } from './pdfPreview';
import { FabricOverlayManager } from './fabricOverlay';
import { fetchPdfBytes, getIR, open, patchDocument } from './api';
import type { DocumentIR } from './types';

const app = document.querySelector<HTMLDivElement>('#app');
if (!app) {
  throw new Error('App container missing');
}

app.innerHTML = `
  <div class="layout">
    <aside class="sidebar">
      <h1>PDF Transform MVP</h1>
      <p class="sidebar__intro">Upload a PDF and drag the overlays to update the underlying content stream matrices.</p>
      <label class="button button--secondary">
        <span>Select PDF…</span>
        <input id="file-input" type="file" accept="application/pdf" hidden />
      </label>
      <button id="download" class="button" disabled>Download current PDF</button>
      <div id="status" class="status"></div>
    </aside>
    <section class="editor">
      <div id="page-stack" class="page-stack"></div>
    </section>
  </div>
`;

const statusEl = document.getElementById('status') as HTMLDivElement;
const pageStack = document.getElementById('page-stack') as HTMLDivElement;
const fileInput = document.getElementById('file-input') as HTMLInputElement;
const downloadButton = document.getElementById('download') as HTMLButtonElement;

const pdfContainer = document.createElement('div');
const preview = new PdfPreview(pdfContainer);
const overlayManager = new FabricOverlayManager();

let currentDocId: string | null = null;
let isBusy = false;

function setStatus(message: string) {
  statusEl.textContent = message;
}

async function renderDocument(docId: string, ir: DocumentIR, pdfData: ArrayBuffer) {
  pageStack.innerHTML = '';
  pdfContainer.innerHTML = '';
  await preview.load(pdfData);
  const sizes = preview.getSizes();
  const canvases = Array.from(pdfContainer.querySelectorAll('canvas'));
  const overlayWrappers: HTMLElement[] = [];

  canvases.forEach((canvas, index) => {
    const wrapper = document.createElement('div');
    wrapper.className = 'page-wrapper';
    wrapper.style.width = `${canvas.width}px`;
    wrapper.style.height = `${canvas.height}px`;
    const pdfLayer = document.createElement('div');
    pdfLayer.className = 'page-wrapper__pdf';
    pdfLayer.appendChild(canvas);
    const overlayLayer = document.createElement('div');
    overlayLayer.className = 'page-wrapper__overlay';
    wrapper.appendChild(pdfLayer);
    wrapper.appendChild(overlayLayer);
    pageStack.appendChild(wrapper);
    overlayWrappers[index] = overlayLayer;
  });

  overlayManager.populate(ir, overlayWrappers, sizes, async (patch) => {
    if (!currentDocId) {
      return false;
    }
    try {
      await patchDocument(currentDocId, [patch]);
      await refreshDocument(currentDocId);
      setStatus('Transform applied.');
      return true;
    } catch (err) {
      console.error(err);
      setStatus(`Transform failed: ${err instanceof Error ? err.message : String(err)}`);
      return false;
    }
  });
}

async function refreshDocument(docId: string) {
  if (isBusy) return;
  isBusy = true;
  try {
    setStatus('Fetching IR and PDF…');
    const [ir, pdfBytes] = await Promise.all([getIR(docId), fetchPdfBytes(docId)]);
    await renderDocument(docId, ir, pdfBytes);
    downloadButton.disabled = false;
    setStatus('Ready. Drag the overlays to update the PDF.');
  } catch (err) {
    console.error(err);
    setStatus(`Failed to load document: ${err instanceof Error ? err.message : String(err)}`);
  } finally {
    isBusy = false;
  }
}

fileInput.addEventListener('change', async (event) => {
  const input = event.target as HTMLInputElement;
  if (!input.files || input.files.length === 0) {
    return;
  }
  const file = input.files[0];
  try {
    setStatus('Uploading PDF…');
    const { docId } = await open(file);
    currentDocId = docId;
    await refreshDocument(docId);
  } catch (err) {
    console.error(err);
    setStatus(`Failed to open PDF: ${err instanceof Error ? err.message : String(err)}`);
  }
});

downloadButton.addEventListener('click', async () => {
  if (!currentDocId || isBusy) {
    return;
  }
  try {
    setStatus('Preparing download…');
    const bytes = await fetchPdfBytes(currentDocId);
    const blob = new Blob([bytes], { type: 'application/pdf' });
    const url = URL.createObjectURL(blob);
    const link = document.createElement('a');
    link.href = url;
    link.download = `${currentDocId}.pdf`;
    link.click();
    URL.revokeObjectURL(url);
    setStatus('Download triggered.');
  } catch (err) {
    console.error(err);
    setStatus(`Download failed: ${err instanceof Error ? err.message : String(err)}`);
  }
});

setStatus('Select a PDF to begin.');
