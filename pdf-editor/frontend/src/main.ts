import './styles.css';
import { openDocument, applyPatch, downloadPdf } from './api';
import type { DocumentIR, PageObject } from './types';
import { renderDocument } from './pdfPreview';
import { FabricOverlay } from './fabricOverlay';

const app = document.querySelector<HTMLDivElement>('#app');
if (!app) {
  throw new Error('App container missing');
}

app.innerHTML = `
  <div class="layout">
    <aside class="sidebar">
      <h1>PDF Editor</h1>
      <label class="file-upload">
        <span>Select PDF</span>
        <input type="file" accept="application/pdf" />
      </label>
      <button id="download-btn" disabled>Download PDF</button>
      <div class="status" id="status"></div>
    </aside>
    <main class="workspace">
      <div id="pages"></div>
    </main>
  </div>
`;

const fileInput = app.querySelector<HTMLInputElement>('input[type="file"]');
const pagesContainer = app.querySelector<HTMLDivElement>('#pages');
const downloadBtn = app.querySelector<HTMLButtonElement>('#download-btn');
const statusEl = app.querySelector<HTMLDivElement>('#status');

let currentDocId: string | null = null;
let currentIR: DocumentIR | null = null;
let currentFileData: Uint8Array | null = null;

const overlay = new FabricOverlay({
  onTransform: async (ops) => {
    if (!currentDocId) return;
    try {
      const response = await applyPatch(currentDocId, ops);
      if (response.ok) {
        status(`Applied transform, received updated PDF (${response.updatedPdf ? 'inline' : 'no data'})`);
      } else {
        status(`Patch failed: ${response.error ?? 'unknown error'}`);
      }
    } catch (err) {
      status(`Patch failed: ${(err as Error).message}`);
    }
  },
  onEditText: async (obj: PageObject) => {
    const text = prompt('Enter replacement text:');
    if (!text) return;
    status(`Text editing is not wired to backend yet for object ${obj.id}.`);
  }
});

function status(message: string) {
  if (statusEl) {
    statusEl.textContent = message;
  }
}

fileInput?.addEventListener('change', async (event) => {
  const files = (event.target as HTMLInputElement).files;
  if (!files || files.length === 0) {
    return;
  }
  const file = files[0];
  const arrayBuffer = await file.arrayBuffer();
  currentFileData = new Uint8Array(arrayBuffer);

  if (!pagesContainer) return;
  pagesContainer.textContent = '';

  const docWrapper = document.createElement('div');
  docWrapper.className = 'page-stack';
  pagesContainer.appendChild(docWrapper);

  try {
    const { docId, ir } = await openDocument(file);
    currentDocId = docId;
    currentIR = ir;

    const pageHosts = new Map<number, HTMLDivElement>();
    ir.pages.forEach((page) => {
      const pageContainer = document.createElement('div');
      pageContainer.id = `page-${page.index}`;
      pageContainer.className = 'page-container';
      pageContainer.style.width = `${page.widthPt * (96 / 72)}px`;
      pageContainer.style.height = `${page.heightPt * (96 / 72)}px`;
      docWrapper.appendChild(pageContainer);
      pageHosts.set(page.index, pageContainer);
    });

    if (currentFileData) {
      await renderDocument(currentFileData, (index) => {
        const host = pageHosts.get(index);
        if (!host) {
          throw new Error(`Missing page container for index ${index}`);
        }
        return host;
      });
    }

    overlay.attachToDocument(docId, ir);
    status(`Loaded document with ${ir.pages.length} pages.`);
    if (downloadBtn) downloadBtn.disabled = false;
  } catch (err) {
    status(`Failed to load PDF: ${(err as Error).message}`);
  }
});

downloadBtn?.addEventListener('click', async () => {
  if (!currentDocId) return;
  try {
    const blob = await downloadPdf(currentDocId);
    const url = URL.createObjectURL(blob);
    const link = document.createElement('a');
    link.href = url;
    link.download = 'edited.pdf';
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
    status('Downloaded latest PDF snapshot.');
  } catch (err) {
    status(`Download failed: ${(err as Error).message}`);
  }
});
