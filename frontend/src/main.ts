import './styles.css';
import { PdfPreview } from './pdfPreview';
import { FabricOverlayManager } from './fabricOverlay';
import * as api from './api';
import type { DocumentIR, PatchOperation } from './types';

const app = document.querySelector<HTMLDivElement>('#app');
if (!app) {
  throw new Error('App container missing');
}

app.innerHTML = `
  <div class="layout">
    <aside class="sidebar">
      <h1>PDF Transform MVP</h1>
      <p class="sidebar__intro">Upload a PDF to inspect page&nbsp;0 and drag controllers to adjust matrices.</p>
      <label class="button">
        <span>Select PDF…</span>
        <input id="file-input" type="file" accept="application/pdf" hidden />
      </label>
      <button id="download" class="button button--secondary" disabled>Download updated PDF</button>
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

fileInput.addEventListener('change', async (event) => {
  const input = event.target as HTMLInputElement;
  if (!input.files || input.files.length === 0) {
    return;
  }
  const file = input.files[0];
  try {
    setStatus(`Uploading ${file.name}…`);
    const { docId } = await api.open(file);
    currentDocId = docId;
    await loadDocument(docId);
    downloadButton.disabled = false;
    setStatus(`Loaded ${file.name}. Drag the controllers to update the PDF.`);
  } catch (err) {
    console.error(err);
    setStatus(`Failed to open PDF: ${err instanceof Error ? err.message : String(err)}`);
  } finally {
    input.value = '';
  }
});

downloadButton.addEventListener('click', () => {
  if (!currentDocId) {
    return;
  }
  void api
    .download(currentDocId)
    .then(() => setStatus('Download started.'))
    .catch((err) => setStatus(`Download failed: ${err instanceof Error ? err.message : String(err)}`));
});

async function loadDocument(docId: string) {
  const ir = await api.getIR(docId);
  const pdfData = await api.fetchPdf(docId);
  await render(ir, pdfData);
}

async function render(ir: DocumentIR, pdfData: ArrayBuffer) {
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

  const docId = currentDocId;
  try {
    await overlayManager.populate(
      ir,
      overlayWrappers,
      sizes,
      async (ops: PatchOperation[]) => {
        if (!docId) {
          throw new Error('Document not loaded');
        }
        const response = await api.patch(docId, ops);
        if (response.updatedPdf) {
          const data = response.updatedPdf.split(',')[1];
          if (data) {
            const pdfBytes = Uint8Array.from(atob(data), (c) => c.charCodeAt(0)).buffer;
            await preview.load(pdfBytes);
          }
        }
        return response;
      },
    );
  } catch (err) {
    setStatus(`Failed to initialise overlay: ${err instanceof Error ? err.message : String(err)}`);
  }
}

function setStatus(message: string) {
  statusEl.textContent = message;
}

setStatus('Select a PDF to begin.');
