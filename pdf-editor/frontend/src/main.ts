import './styles.css';
import { openPdf, fetchIr } from './api';
import { renderPdf } from './pdfPreview';
import { createOverlay, syncOverlay } from './fabricOverlay';
import type { IrDocument } from './types';

const app = document.querySelector<HTMLDivElement>('#app');
if (!app) {
  throw new Error('Missing #app container');
}

app.innerHTML = `
  <aside class="sidebar">
    <h1>PDF Editor</h1>
    <input type="file" id="file-input" accept="application/pdf" />
    <div class="toolbar">
      <button id="download-btn" disabled>Download PDF</button>
    </div>
    <div id="status"></div>
  </aside>
  <main class="editor">
    <div id="preview-container"></div>
    <canvas id="overlay-canvas"></canvas>
  </main>
`;

const fileInput = document.querySelector<HTMLInputElement>('#file-input');
const previewContainer = document.querySelector<HTMLDivElement>('#preview-container');
const overlayCanvas = document.querySelector<HTMLCanvasElement>('#overlay-canvas');
const statusEl = document.querySelector<HTMLDivElement>('#status');

if (!fileInput || !previewContainer || !overlayCanvas || !statusEl) {
  throw new Error('Failed to initialise UI');
}

const overlay = createOverlay(overlayCanvas);
let activeDocId: string | null = null;
let irDocument: IrDocument | null = null;

fileInput.addEventListener('change', async (event) => {
  const files = (event.target as HTMLInputElement).files;
  if (!files || files.length === 0) {
    return;
  }
  const file = files[0];
  statusEl.textContent = 'Uploading…';
  try {
    const openResponse = await openPdf(file);
    activeDocId = openResponse.docId;
    statusEl.textContent = `Opened document ${activeDocId}`;
    const ir = await fetchIr(activeDocId);
    irDocument = ir;
    const buffer = await file.arrayBuffer();
    await renderPdf(previewContainer, buffer);
    if (ir.pages.length > 0) {
      const firstPage = ir.pages[0];
      syncOverlay(overlay, firstPage, firstPage.objects);
    }
  } catch (error) {
    console.error(error);
    statusEl.textContent = 'Failed to open PDF';
  }
});
