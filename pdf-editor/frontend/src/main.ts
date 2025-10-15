import './styles.css';
import { openDocument, sendPatch } from './api';
import type { PatchOp, DocumentIR } from './types';
import { FabricOverlay } from './fabricOverlay';
import { renderPdfPreview } from './pdfPreview';

const app = document.getElementById('app');
if (!app) {
  throw new Error('#app not found');
}

app.innerHTML = `
  <main class="layout">
    <header class="toolbar">
      <input type="file" id="file-input" accept="application/pdf" />
      <button id="download-btn" disabled>Download PDF</button>
      <span id="status"></span>
    </header>
    <section class="content">
      <div class="preview" id="preview"></div>
      <div class="overlay" id="overlay"></div>
    </section>
  </main>
`;

const fileInput = document.getElementById('file-input') as HTMLInputElement;
const downloadBtn = document.getElementById('download-btn') as HTMLButtonElement;
const statusEl = document.getElementById('status') as HTMLSpanElement;
const previewEl = document.getElementById('preview') as HTMLDivElement;
const overlayEl = document.getElementById('overlay') as HTMLDivElement;

let currentDocId: string | null = null;
let currentIR: DocumentIR | null = null;
let overlay: FabricOverlay | null = null;

fileInput.addEventListener('change', async (event) => {
  const files = (event.target as HTMLInputElement).files;
  if (!files || files.length === 0) return;
  const file = files[0];
  statusEl.textContent = 'Opening…';
  try {
    const { docId, ir } = await openDocument(file);
    currentDocId = docId;
    currentIR = ir;
    const arrayBuffer = await file.arrayBuffer();
    await renderPdfPreview({ container: previewEl, data: new Uint8Array(arrayBuffer) });
    overlay = new FabricOverlay({
      container: overlayEl,
      ir,
      onPatch: async (ops: PatchOp[]) => {
        if (!currentDocId) return;
        const response = await sendPatch(currentDocId, ops);
        statusEl.textContent = response.ok ? 'Saved' : response.message ?? 'Patch failed';
      },
    });
    statusEl.textContent = 'Ready';
    downloadBtn.disabled = false;
  } catch (error) {
    console.error(error);
    statusEl.textContent = 'Failed to open PDF';
  }
});

// Download stub
if (downloadBtn) {
  downloadBtn.addEventListener('click', async () => {
    if (!currentDocId) return;
    statusEl.textContent = 'Downloading…';
    try {
      const blob = await fetch(`${import.meta.env.VITE_API_BASE ?? 'http://localhost:8787'}/api/pdf/${currentDocId}`).then((r) => r.blob());
      const url = URL.createObjectURL(blob);
      const anchor = document.createElement('a');
      anchor.href = url;
      anchor.download = 'edited.pdf';
      anchor.click();
      URL.revokeObjectURL(url);
      statusEl.textContent = 'Download ready';
    } catch (error) {
      console.error(error);
      statusEl.textContent = 'Download failed';
    }
  });
}
