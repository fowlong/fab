import './styles.css';
import { PdfPreview } from './pdfPreview';
import { FabricOverlayManager } from './fabricOverlay';
import { download, getIR, getPdfBytes, open, patch } from './api';
import type { PatchOperation } from './types';

const app = document.querySelector<HTMLDivElement>('#app');
if (!app) {
  throw new Error('App container missing');
}

app.innerHTML = `
  <div class="layout">
    <aside class="sidebar">
      <h1>PDF transformer</h1>
      <p class="sidebar__intro">Upload a PDF to reposition text runs or image XObjects. Drag the overlay handles to move, rotate, or scale.</p>
      <label class="button button--secondary">
        <span>Select PDF…</span>
        <input id="file-input" type="file" accept="application/pdf" hidden />
      </label>
      <button id="download" class="button" disabled>Download updated PDF</button>
      <div id="status" class="status"></div>
    </aside>
    <section class="editor">
      <div id="page-wrapper" class="page-wrapper">
        <div id="pdf-layer" class="page-wrapper__pdf"></div>
        <div id="overlay-layer" class="page-wrapper__overlay"></div>
      </div>
    </section>
  </div>
`;

const statusEl = document.getElementById('status') as HTMLDivElement;
const fileInput = document.getElementById('file-input') as HTMLInputElement;
const downloadButton = document.getElementById('download') as HTMLButtonElement;
const pdfLayer = document.getElementById('pdf-layer') as HTMLDivElement;
const overlayLayer = document.getElementById('overlay-layer') as HTMLDivElement;

const preview = new PdfPreview(pdfLayer);
const overlay = new FabricOverlayManager();

let currentDocId: string | null = null;
let pending = false;

fileInput.addEventListener('change', async (event) => {
  const target = event.target as HTMLInputElement;
  if (!target.files || target.files.length === 0) {
    return;
  }
  const file = target.files[0];
  await loadDocument(file);
  target.value = '';
});

downloadButton.addEventListener('click', () => {
  if (!currentDocId) {
    return;
  }
  download(currentDocId).catch((error) => {
    setStatus(`Failed to download PDF: ${String(error)}`);
  });
});

async function loadDocument(file: File) {
  if (pending) return;
  pending = true;
  setStatus('Uploading PDF…');
  try {
    const { docId } = await open(file);
    currentDocId = docId;
    await refreshView();
    downloadButton.disabled = false;
    setStatus(`Loaded ${file.name}`);
  } catch (error) {
    setStatus(`Failed to load document: ${String(error)}`);
  } finally {
    pending = false;
  }
}

async function refreshView() {
  if (!currentDocId) {
    return;
  }
  setStatus('Rendering page…');
  const [buffer, ir] = await Promise.all([
    getPdfBytes(currentDocId),
    getIR(currentDocId),
  ]);
  const page = ir.pages[0];
  const render = await preview.renderFirstPage(buffer);
  overlay.mount(overlayLayer, render.widthPx, render.heightPx);
  overlay.render(page, async ({ id, kind, delta }) => {
    if (!currentDocId) {
      return;
    }
    await applyTransform({
      op: 'transform',
      target: { page: 0, id },
      deltaMatrixPt: delta,
      kind,
    });
  });
  setStatus('Ready. Drag an overlay to transform the PDF.');
}

async function applyTransform(operation: PatchOperation) {
  if (!currentDocId || pending) {
    return;
  }
  pending = true;
  setStatus('Applying transform…');
  try {
    await patch(currentDocId, [operation]);
    await refreshView();
    setStatus('Transform applied.');
  } catch (error) {
    setStatus(`Failed to apply transform: ${String(error)}`);
  } finally {
    pending = false;
  }
}

function setStatus(message: string) {
  statusEl.textContent = message;
}

setStatus('Select a PDF to begin.');
