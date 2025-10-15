import './styles.css';
import { initialisePdfPreview } from './pdfPreview';
import { initialiseFabricOverlay } from './fabricOverlay';
import { ApiClient } from './api';

const app = document.querySelector<HTMLDivElement>('#app');

if (!app) {
  throw new Error('Application root element not found');
}

app.innerHTML = `
  <main class="layout">
    <aside class="sidebar">
      <h1>PDF Editor</h1>
      <input type="file" id="file-input" accept="application/pdf" />
      <button id="download-btn" disabled>Download PDF</button>
      <div id="status"></div>
    </aside>
    <section class="editor">
      <div id="page-container"></div>
    </section>
  </main>
`;

const fileInput = document.querySelector<HTMLInputElement>('#file-input');
const downloadBtn = document.querySelector<HTMLButtonElement>('#download-btn');
const statusEl = document.querySelector<HTMLDivElement>('#status');
const pageContainer = document.querySelector<HTMLDivElement>('#page-container');

if (!fileInput || !downloadBtn || !statusEl || !pageContainer) {
  throw new Error('Expected layout elements');
}

const api = new ApiClient();

let docId: string | null = null;

fileInput.addEventListener('change', async (event) => {
  const target = event.target as HTMLInputElement;
  const file = target.files?.[0];
  if (!file) {
    return;
  }

  statusEl.textContent = 'Uploading…';
  try {
    const openResult = await api.open(file);
    docId = openResult.docId;
    statusEl.textContent = 'Loading document…';
    const ir = await api.fetchIR(docId);
    statusEl.textContent = 'Rendering…';

    pageContainer.innerHTML = '';
    await initialisePdfPreview(ir, pageContainer);
    initialiseFabricOverlay(ir, pageContainer, {
      onPatch: async (ops) => {
        if (!docId) return;
        const response = await api.patch(docId, ops);
        if (response.ok) {
          statusEl.textContent = 'Saved changes';
        }
        return response;
      },
    });

    downloadBtn.disabled = false;
    statusEl.textContent = 'Ready';
  } catch (error) {
    console.error(error);
    statusEl.textContent = 'Failed to open document';
  }
});

downloadBtn.addEventListener('click', async () => {
  if (!docId) return;
  const blob = await api.download(docId);
  const url = URL.createObjectURL(blob);
  const anchor = document.createElement('a');
  anchor.href = url;
  anchor.download = 'edited.pdf';
  anchor.click();
  URL.revokeObjectURL(url);
});
