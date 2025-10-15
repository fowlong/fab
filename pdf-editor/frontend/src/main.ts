import './styles.css';
import { openDocument, sendPatch, downloadPdf } from './api';
import { renderPdfPreview } from './pdfPreview';
import { createFabricOverlay } from './fabricOverlay';
import type { PatchOp } from './types';

const app = document.querySelector<HTMLDivElement>('#app');
if (!app) {
  throw new Error('App root not found');
}

const layout = document.createElement('div');
layout.className = 'layout';
layout.innerHTML = `
  <aside class="sidebar">
    <input type="file" accept="application/pdf" />
    <button data-action="download" disabled>Download PDF</button>
    <div class="status"></div>
  </aside>
  <main class="editor">
    <div class="pages"></div>
  </main>
`;
app.appendChild(layout);

const fileInput = layout.querySelector<HTMLInputElement>('input[type="file"]');
const downloadButton = layout.querySelector<HTMLButtonElement>('button[data-action="download"]');
const statusBox = layout.querySelector<HTMLDivElement>('.status');
const pagesContainer = layout.querySelector<HTMLDivElement>('.pages');

if (!fileInput || !downloadButton || !statusBox || !pagesContainer) {
  throw new Error('Missing UI element');
}

let currentDocId: string | null = null;
const overlay = createFabricOverlay();
overlay.setPatchHandler(async (ops: PatchOp[]) => {
  if (!currentDocId) return;
  setStatus('Sending patch...');
  const resp = await sendPatch(currentDocId, ops);
  if (!resp.ok) {
    setStatus(`Patch failed: ${resp.error ?? 'unknown error'}`);
    return;
  }
  setStatus('Patch applied');
});

fileInput.addEventListener('change', async () => {
  const [file] = fileInput.files ?? [];
  if (!file) return;
  setStatus('Opening PDF...');
  try {
    const { docId, pages } = await openDocument(file);
    currentDocId = docId;
    const arrayBuffer = await file.arrayBuffer();
    const { pages: previewPages } = await renderPdfPreview(arrayBuffer, pagesContainer);
    overlay.destroy();
    previewPages.forEach((pagePreview, index) => {
      const canvas = overlay.mount(pagePreview.container, pages[index]);
      canvas.setWidth(pagePreview.canvas.width);
      canvas.setHeight(pagePreview.canvas.height);
    });
    overlay.update({ pages });
    downloadButton.disabled = false;
    setStatus('PDF loaded');
  } catch (err) {
    console.error(err);
    setStatus('Failed to load PDF');
  }
});

downloadButton.addEventListener('click', async () => {
  if (!currentDocId) return;
  try {
    const blob = await downloadPdf(currentDocId);
    const url = URL.createObjectURL(blob);
    const anchor = document.createElement('a');
    anchor.href = url;
    anchor.download = 'edited.pdf';
    anchor.click();
    URL.revokeObjectURL(url);
  } catch (err) {
    console.error(err);
    setStatus('Download failed');
  }
});

function setStatus(message: string) {
  statusBox.textContent = message;
}
