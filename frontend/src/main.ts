import './styles.css';
import { initialisePdfPreview } from './pdfPreview';
import { initialiseFabricOverlay } from './fabricOverlay';
import { loadInitialDocument } from './api';

const app = document.getElementById('app');
if (!app) {
  throw new Error('Missing #app container');
}

app.innerHTML = `
  <main class="layout">
    <aside class="sidebar">
      <h1>PDF Editor</h1>
      <input id="file-input" type="file" accept="application/pdf" />
      <button id="download-button" disabled>Download PDF</button>
      <div id="status"></div>
    </aside>
    <section class="editor">
      <div id="page-container" class="page-stack"></div>
    </section>
  </main>
`;

const fileInput = document.getElementById('file-input') as HTMLInputElement;
const downloadButton = document.getElementById('download-button') as HTMLButtonElement;
const status = document.getElementById('status') as HTMLDivElement;
const pageContainer = document.getElementById('page-container') as HTMLDivElement;

let currentDocId: string | null = null;

fileInput.addEventListener('change', async () => {
  const file = fileInput.files?.[0];
  if (!file) {
    return;
  }
  status.textContent = 'Uploading…';
  try {
    const { docId, ir } = await loadInitialDocument(file);
    currentDocId = docId;
    downloadButton.disabled = false;
    status.textContent = 'Loaded document';
    pageContainer.innerHTML = '';
    await initialisePdfPreview(pageContainer, ir);
    initialiseFabricOverlay(pageContainer, ir, docId);
  } catch (error) {
    console.error(error);
    status.textContent = 'Failed to load document';
  }
});

if (downloadButton) {
  downloadButton.addEventListener('click', async () => {
    if (!currentDocId) {
      return;
    }
    try {
      const response = await fetch(`/api/pdf/${currentDocId}`);
      if (!response.ok) {
        throw new Error('Failed to download PDF');
      }
      const blob = await response.blob();
      const url = URL.createObjectURL(blob);
      const link = document.createElement('a');
      link.href = url;
      link.download = 'edited.pdf';
      link.click();
      URL.revokeObjectURL(url);
    } catch (error) {
      console.error(error);
    }
  });
}
