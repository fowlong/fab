import './styles.css';

import { open, getIR, patch, fetchPdfBytes, download } from './api';
import { PdfPreview } from './pdfPreview';
import { FabricOverlay } from './fabricOverlay';

const app = document.querySelector<HTMLDivElement>('#app');
if (!app) {
  throw new Error('App container missing');
}

app.innerHTML = `
  <div class="layout">
    <aside class="sidebar">
      <h1>PDF Transform MVP</h1>
      <label class="button">
        <span>Select PDF…</span>
        <input id="file-input" type="file" accept="application/pdf" hidden />
      </label>
      <button id="download-btn" class="button button--secondary" disabled>Download updated PDF</button>
      <div id="status" class="status"></div>
    </aside>
    <main class="editor">
      <div id="viewer" class="viewer">
        <div id="pdf-layer" class="viewer__pdf"></div>
        <div id="overlay-layer" class="viewer__overlay"></div>
      </div>
    </main>
  </div>
`;

const statusEl = document.getElementById('status') as HTMLDivElement;
const fileInput = document.getElementById('file-input') as HTMLInputElement;
const downloadButton = document.getElementById('download-btn') as HTMLButtonElement;
const pdfLayer = document.getElementById('pdf-layer') as HTMLDivElement;
const overlayLayer = document.getElementById('overlay-layer') as HTMLDivElement;

const preview = new PdfPreview(pdfLayer);
let overlay: FabricOverlay | null = null;
let currentDocId: string | null = null;
let isBusy = false;

fileInput.addEventListener('change', async (event) => {
  const input = event.target as HTMLInputElement;
  if (!input.files || input.files.length === 0) {
    return;
  }
  const file = input.files[0];
  try {
    setStatus(`Uploading ${file.name}…`);
    isBusy = true;
    const response = await open(file);
    currentDocId = response.docId;
    await renderDocument();
  } catch (error) {
    reportError('Failed to open PDF', error);
  } finally {
    isBusy = false;
    fileInput.value = '';
  }
});

downloadButton.addEventListener('click', async () => {
  if (!currentDocId || isBusy) {
    return;
  }
  try {
    setStatus('Preparing download…');
    await download(currentDocId);
    setStatus('Download ready.');
  } catch (error) {
    reportError('Download failed', error);
  }
});

async function renderDocument() {
  if (!currentDocId) {
    return;
  }
  try {
    isBusy = true;
    setStatus('Fetching IR…');
    const ir = await getIR(currentDocId);
    const page = ir.pages[0];
    setStatus('Rendering PDF…');
    const pdfBytes = await fetchPdfBytes(currentDocId);
    const renderResult = await preview.renderPage0(pdfBytes);

    overlayLayer.innerHTML = '';
    overlayLayer.style.width = `${renderResult.width}px`;
    overlayLayer.style.height = `${renderResult.height}px`;

    const overlayCanvas = document.createElement('canvas');
    overlayCanvas.width = renderResult.width;
    overlayCanvas.height = renderResult.height;
    overlayCanvas.className = 'pdf-overlay';
    overlayLayer.appendChild(overlayCanvas);

    overlay?.dispose();
    overlay = new FabricOverlay(overlayCanvas, page, async ({ id, kind, delta }) => {
      if (!currentDocId) {
        return;
      }
      setStatus('Applying transform…');
      try {
        await patch(currentDocId, [
          {
            op: 'transform',
            target: { page: 0, id },
            deltaMatrixPt: delta,
            kind,
          },
        ]);
        await renderDocument();
        setStatus('Transform applied.');
      } catch (error) {
        reportError('Transform failed', error);
        throw error;
      }
    });
    overlay.populate(ir);
    downloadButton.disabled = false;
    setStatus('Ready.');
  } catch (error) {
    reportError('Failed to render document', error);
  } finally {
    isBusy = false;
  }
}

function setStatus(message: string) {
  statusEl.textContent = message;
}

function reportError(prefix: string, error: unknown) {
  const message = error instanceof Error ? error.message : String(error);
  setStatus(`${prefix}: ${message}`);
  console.error(prefix, error);
}
