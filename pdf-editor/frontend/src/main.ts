import './styles.css';
import { initPdfPreview } from './pdfPreview';
import { initFabricOverlay } from './fabricOverlay';
import { loadSampleDocument, openFile, type OpenResponse } from './api';

async function bootstrap() {
  const root = document.querySelector<HTMLDivElement>('#app');
  if (!root) {
    throw new Error('Missing #app container');
  }

  root.innerHTML = `
    <main class="layout">
      <aside class="sidebar">
        <h1>PDF Editor</h1>
        <label class="upload">
          <span>Open PDF</span>
          <input type="file" id="file-input" accept="application/pdf" />
        </label>
        <button id="open-sample">Open Sample</button>
        <button id="download-btn" disabled>Download PDF</button>
        <div id="status"></div>
      </aside>
      <section class="canvas-container">
        <div id="page-stack"></div>
      </section>
    </main>
  `;

  const pageStack = document.querySelector<HTMLDivElement>('#page-stack');
  if (!pageStack) {
    throw new Error('Missing #page-stack container');
  }

  const status = document.querySelector<HTMLDivElement>('#status');
  status?.classList.add('status');

  const fileInput = document.querySelector<HTMLInputElement>('#file-input');
  const sampleBtn = document.querySelector<HTMLButtonElement>('#open-sample');
  const downloadBtn = document.querySelector<HTMLButtonElement>('#download-btn');

  let overlayHandle: ReturnType<typeof initFabricOverlay> | null = null;

  async function loadDoc(loader: () => Promise<OpenResponse | null>) {
    status && (status.textContent = 'Loading document…');
    const doc = await loader();
    if (!doc) {
      status && (status.textContent = 'Failed to load document');
      if (downloadBtn) {
        downloadBtn.disabled = true;
      }
      overlayHandle = null;
      return;
    }

    pageStack.replaceChildren();
    const { docId, ir } = doc;
    const preview = await initPdfPreview(ir.pages, pageStack, docId);
    overlayHandle = initFabricOverlay({ pages: ir.pages, docId, preview });
    if (downloadBtn) {
      downloadBtn.disabled = false;
    }
    status && (status.textContent = 'Document ready');
  }

  sampleBtn?.addEventListener('click', () => {
    void loadDoc(loadSampleDocument);
  });

  fileInput?.addEventListener('change', () => {
    const file = fileInput.files?.[0];
    if (!file) {
      return;
    }
    void loadDoc(() => openFile(file));
  });

  downloadBtn?.addEventListener('click', async () => {
    if (!overlayHandle) {
      return;
    }
    const blob = await overlayHandle.downloadLatest();
    if (!blob) {
      status && (status.textContent = 'Failed to download PDF');
      return;
    }

    const url = URL.createObjectURL(blob);
    const link = document.createElement('a');
    link.href = url;
    link.download = 'edited.pdf';
    link.click();
    URL.revokeObjectURL(url);
  });

  await loadDoc(loadSampleDocument);
}

void bootstrap();
