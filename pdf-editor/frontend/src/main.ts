import './styles.css';
import { initialisePdfPreview } from './pdfPreview';
import { initialiseFabricOverlay } from './fabricOverlay';
import { ApiClient } from './api';

declare const __API_BASE__: string;

const app = document.querySelector<HTMLDivElement>('#app');
if (!app) {
  throw new Error('Missing #app root element');
}

app.innerHTML = `
  <div class="layout">
    <aside class="sidebar">
      <h1>PDF Editor</h1>
      <div class="controls">
        <label class="file-upload">Open PDF
          <input id="file-input" type="file" accept="application/pdf" />
        </label>
        <button id="download-btn" disabled>Download PDF</button>
        <div id="status"></div>
      </div>
    </aside>
    <main class="editor">
      <div id="page-container" class="page-container"></div>
    </main>
  </div>
`;

const fileInput = document.querySelector<HTMLInputElement>('#file-input');
const downloadBtn = document.querySelector<HTMLButtonElement>('#download-btn');
const statusEl = document.querySelector<HTMLDivElement>('#status');
const pageContainer = document.querySelector<HTMLDivElement>('#page-container');

if (!fileInput || !downloadBtn || !statusEl || !pageContainer) {
  throw new Error('Editor layout failed to initialise');
}

const api = new ApiClient(__API_BASE__);
let currentDocId: string | null = null;

async function openFile(file: File) {
  statusEl.textContent = 'Uploading PDF…';
  try {
    const docId = await api.open(file);
    currentDocId = docId;
    statusEl.textContent = 'Fetching document structure…';
    const ir = await api.fetchIr(docId);
    statusEl.textContent = 'Rendering preview…';
    const preview = await initialisePdfPreview(pageContainer, ir);
    initialiseFabricOverlay(preview, ir, async (ops) => {
      if (!currentDocId) {
        return;
      }
      statusEl.textContent = 'Applying patch…';
      const result = await api.patch(currentDocId, ops);
      if (result.updatedPdf) {
        downloadBtn.disabled = false;
      }
      statusEl.textContent = 'Saved';
      return result;
    });
    downloadBtn.disabled = false;
    statusEl.textContent = 'Ready';
  } catch (error) {
    console.error(error);
    statusEl.textContent = 'Failed to open PDF';
  }
}

fileInput.addEventListener('change', () => {
  const file = fileInput.files?.[0];
  if (file) {
    void openFile(file);
  }
});

downloadBtn.addEventListener('click', async () => {
  if (!currentDocId) {
    return;
  }
  try {
    const blob = await api.download(currentDocId);
    const url = URL.createObjectURL(blob);
    const anchor = document.createElement('a');
    anchor.href = url;
    anchor.download = 'edited.pdf';
    anchor.click();
    URL.revokeObjectURL(url);
  } catch (error) {
    console.error(error);
    statusEl.textContent = 'Download failed';
  }
});
