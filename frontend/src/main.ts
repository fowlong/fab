import './styles.css';
import { PdfPreview } from './pdfPreview';
import { FabricOverlayManager, type TransformHandler } from './fabricOverlay';
import * as api from './api';
import type { DocumentIR, PageIR, TransformPatch } from './types';

const app = document.querySelector<HTMLDivElement>('#app');
if (!app) {
  throw new Error('App container missing');
}

type AppState = {
  docId: string | null;
  ir: DocumentIR | null;
};

const state: AppState = { docId: null, ir: null };

app.innerHTML = `
  <div class="layout">
    <aside class="sidebar">
      <h1>PDF Transform MVP</h1>
      <p class="sidebar__intro">Upload a PDF, adjust the overlays, and the backend will rewrite the content stream incrementally.</p>
      <label class="button">
        <span>Select PDF…</span>
        <input id="file-input" type="file" accept="application/pdf" hidden />
      </label>
      <button id="download" class="button button--secondary" disabled>Download current PDF</button>
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
const pdfLayer = document.getElementById('pdf-layer') as HTMLDivElement;
const overlayLayer = document.getElementById('overlay-layer') as HTMLDivElement;
const fileInput = document.getElementById('file-input') as HTMLInputElement;
const downloadButton = document.getElementById('download') as HTMLButtonElement;

const preview = new PdfPreview(pdfLayer);
const overlay = new FabricOverlayManager();

fileInput.addEventListener('change', async (event) => {
  const input = event.target as HTMLInputElement;
  if (!input.files || input.files.length === 0) {
    return;
  }
  const file = input.files[0];
  input.value = '';
  try {
    setStatus(`Uploading “${file.name}”…`);
    const { docId } = await api.open(file);
    state.docId = docId;
    await refreshView();
    setStatus(`Loaded ${file.name}.`);
    downloadButton.disabled = false;
  } catch (err) {
    setStatus(`Upload failed: ${describeError(err)}`);
  }
});

downloadButton.addEventListener('click', async () => {
  if (!state.docId) {
    return;
  }
  try {
    await api.download(state.docId);
    setStatus('Download started.');
  } catch (err) {
    setStatus(`Download failed: ${describeError(err)}`);
  }
});

async function refreshView() {
  if (!state.docId) {
    return;
  }
  setStatus('Fetching IR…');
  state.ir = await api.getIR(state.docId);
  const page = state.ir.pages[0];
  if (!page) {
    throw new Error('Page 0 missing from IR');
  }
  setStatus('Rendering PDF…');
  const pdfBytes = await api.fetchPdfBytes(state.docId);
  await preview.load(pdfBytes);
  const canvas = preview.getCanvas();
  if (!canvas) {
    throw new Error('PDF canvas missing');
  }
  overlay.mount(overlayLayer, canvas.width, canvas.height);
  await overlay.render(page, transformHandler(page));
  setStatus('Ready. Drag the overlays to transform the PDF.');
}

function transformHandler(page: PageIR): TransformHandler {
  return async (id, kind, deltaMatrixPt) => {
    if (!state.docId) {
      return false;
    }
    try {
      const op: TransformPatch = {
        op: 'transform',
        target: { page: page.index, id },
        deltaMatrixPt,
        kind,
      };
      const response = await api.patch(state.docId, [op]);
      if (!response.ok) {
        setStatus('Patch rejected by backend.');
        return false;
      }
      await refreshView();
      return true;
    } catch (err) {
      setStatus(`Patch failed: ${describeError(err)}`);
      return false;
    }
  };
}

function setStatus(message: string) {
  statusEl.textContent = message;
}

function describeError(err: unknown) {
  if (err instanceof Error) {
    return err.message;
  }
  return String(err);
}

setStatus('Select a PDF to begin.');
