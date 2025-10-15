import './styles.css';
import { open, getIR, download } from './api';
import { PdfPreview } from './pdfPreview';
import { FabricOverlay } from './fabricOverlay';
import type { DocumentIR } from './types';

const app = document.querySelector<HTMLDivElement>('#app');
if (!app) {
  throw new Error('App container missing');
}

app.innerHTML = `
  <div class="layout">
    <aside class="sidebar">
      <h1>PDF Transform MVP</h1>
      <p class="sidebar__intro">Upload a PDF and manipulate text or images via matrix transforms.</p>
      <label class="button">
        <span>Select PDF…</span>
        <input id="file-input" type="file" accept="application/pdf" />
      </label>
      <button id="download-btn" class="button button--secondary" disabled>Download updated PDF</button>
      <div id="status" class="status"></div>
    </aside>
    <section class="editor">
      <div class="page-stack">
        <div class="page-wrapper" id="page-wrapper">
          <div class="page-wrapper__pdf" id="pdf-layer"></div>
          <div class="page-wrapper__overlay" id="overlay-layer"></div>
        </div>
      </div>
    </section>
  </div>
`;

const statusEl = document.getElementById('status') as HTMLDivElement;
const pdfLayer = document.getElementById('pdf-layer') as HTMLDivElement;
const overlayLayer = document.getElementById('overlay-layer') as HTMLDivElement;

const pdfCanvas = document.createElement('canvas');
pdfCanvas.className = 'pdf-underlay';
pdfLayer.appendChild(pdfCanvas);

const overlayCanvas = document.createElement('canvas');
overlayCanvas.className = 'fabric-page-overlay';
overlayLayer.appendChild(overlayCanvas);

const preview = new PdfPreview(pdfCanvas);
const overlay = new FabricOverlay({
  onUpdatedPdf: (dataUrl) => {
    if (dataUrl && currentDocId) {
      refreshFromDataUrl(dataUrl).catch((err) => setStatus(`Failed to refresh preview: ${err}`));
    }
  },
  onError: (message) => setStatus(`Error: ${message}`),
  onInfo: (message) => setStatus(message),
});

let currentDocId: string | null = null;

const fileInput = document.getElementById('file-input') as HTMLInputElement;
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
    await loadDocument(docId);
    setStatus(`Document ${docId} ready.`);
    setDownloadEnabled(true);
  } catch (error) {
    setStatus(`Failed to open PDF: ${error}`);
  }
});

const downloadButton = document.getElementById('download-btn') as HTMLButtonElement;
downloadButton.addEventListener('click', async () => {
  if (!currentDocId) return;
  try {
    setStatus('Preparing download…');
    const blob = await download(currentDocId);
    const url = URL.createObjectURL(blob);
    const anchor = document.createElement('a');
    anchor.href = url;
    anchor.download = `${currentDocId}.pdf`;
    anchor.click();
    URL.revokeObjectURL(url);
    setStatus('Download started.');
  } catch (error) {
    setStatus(`Download failed: ${error}`);
  }
});

async function loadDocument(docId: string) {
  const [ir, blob] = await Promise.all([getIR(docId), download(docId)]);
  await renderDocument(docId, ir, await blob.arrayBuffer());
}

async function refreshFromDataUrl(dataUrl: string) {
  if (!currentDocId) return;
  const buffer = await fetch(dataUrl).then((response) => response.arrayBuffer());
  const ir = await getIR(currentDocId);
  await renderDocument(currentDocId, ir, buffer);
}

async function renderDocument(docId: string, ir: DocumentIR, buffer: ArrayBuffer) {
  const size = await preview.load(buffer);
  overlay.mount(
    overlayCanvas,
    { width: size.widthPx, height: size.heightPx },
    docId,
    size.heightPt,
    ir,
  );
}

function setStatus(message: string) {
  statusEl.textContent = message;
}

function setDownloadEnabled(enabled: boolean) {
  downloadButton.disabled = !enabled;
}

setStatus('Select a PDF to begin.');
