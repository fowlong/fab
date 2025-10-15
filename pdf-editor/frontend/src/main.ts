import './styles.css';
import { ApiClient } from './api';
import { FabricOverlayManager } from './fabricOverlay';
import { renderPdfInto } from './pdfPreview';
import type { DocumentIR } from './types';

const app = document.querySelector<HTMLDivElement>('#app');
if (!app) {
  throw new Error('App container missing');
}

app.innerHTML = `
  <main class="layout">
    <aside class="sidebar">
      <h1>PDF Editor</h1>
      <input type="file" accept="application/pdf" id="file-input" />
      <button id="download-btn" disabled>Download PDF</button>
      <div id="status"></div>
    </aside>
    <section class="editor">
      <div id="pdf-container" class="pdf-container"></div>
      <div id="fabric-container" class="fabric-container"></div>
    </section>
  </main>
`;

const fileInput = document.querySelector<HTMLInputElement>('#file-input');
const downloadBtn = document.querySelector<HTMLButtonElement>('#download-btn');
const statusEl = document.querySelector<HTMLDivElement>('#status');
const pdfContainer = document.querySelector<HTMLDivElement>('#pdf-container');
const fabricContainer = document.querySelector<HTMLDivElement>('#fabric-container');

if (!fileInput || !downloadBtn || !statusEl || !pdfContainer || !fabricContainer) {
  throw new Error('UI initialisation failed');
}

const api = new ApiClient({ baseUrl: '/api' });
let currentDocId: string | undefined;
let currentIr: DocumentIR | undefined;
const overlayManager = new FabricOverlayManager(fabricContainer, (ops) => {
  if (!currentDocId) return;
  api
    .patch(currentDocId, ops)
    .then(() => setStatus('Patch applied'))
    .catch((err) => setStatus(`Patch failed: ${String(err)}`));
});

fileInput.addEventListener('change', async (event) => {
  const file = (event.target as HTMLInputElement).files?.[0];
  if (!file) return;
  try {
    setStatus('Uploading PDF…');
    const { docId } = await api.open(file);
    currentDocId = docId;
    setStatus('Parsing…');
    currentIr = await api.getIR(docId);
    await renderPdf(file);
    overlayManager.mount(currentIr);
    downloadBtn.disabled = false;
    setStatus('Ready');
  } catch (err) {
    setStatus(`Failed to load PDF: ${String(err)}`);
  }
});

downloadBtn.addEventListener('click', async () => {
  if (!currentDocId) return;
  try {
    const blob = await api.download(currentDocId);
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = 'edited.pdf';
    a.click();
    URL.revokeObjectURL(url);
    setStatus('Download started');
  } catch (err) {
    setStatus(`Download failed: ${String(err)}`);
  }
});

async function renderPdf(file: File): Promise<void> {
  const buffer = await file.arrayBuffer();
  await renderPdfInto(pdfContainer, new Uint8Array(buffer));
}

function setStatus(message: string): void {
  statusEl.textContent = message;
}
