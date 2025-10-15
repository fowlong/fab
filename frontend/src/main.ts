import './styles.css';
import { PdfPreview } from './pdfPreview';
import { FabricOverlayManager } from './fabricOverlay';
import * as api from './api';
import type { DocumentIR } from './types';

const app = document.querySelector<HTMLDivElement>('#app');
if (!app) {
  throw new Error('App container missing');
}

type PageSize = {
  width: number;
  height: number;
};

app.innerHTML = `
  <div class="layout">
    <aside class="sidebar">
      <h1>PDF Transform MVP</h1>
      <p class="sidebar__intro">Upload a PDF to edit the placement of text runs and images.</p>
      <label class="button">
        <span>Select PDF…</span>
        <input id="file-input" type="file" accept="application/pdf" hidden />
      </label>
      <button id="download" class="button button--secondary" disabled>Download current PDF</button>
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
let currentDocId: string | null = null;
let currentIR: DocumentIR | null = null;

const overlayManager = new FabricOverlayManager(async (meta, delta) => {
  if (!currentDocId) {
    return false;
  }
  const op = {
    op: 'transform',
    target: { page: meta.pageIndex, id: meta.id },
    deltaMatrixPt: delta,
    kind: meta.kind,
  } as const;
  try {
    const response = await api.patch(currentDocId, [op]);
    if (response.updatedPdf) {
      await reloadPreviewFromDataUrl(response.updatedPdf);
      await refreshIR();
      renderOverlay();
    }
    setStatus('Transform applied.');
    return true;
  } catch (error) {
    setStatus(`Failed to apply transform: ${error}`);
    return false;
  }
});

fileInput.addEventListener('change', async (event) => {
  const input = event.target as HTMLInputElement;
  if (!input.files || input.files.length === 0) {
    return;
  }
  const file = input.files[0];
  try {
    setStatus('Uploading…');
    const { docId } = await api.open(file);
    currentDocId = docId;
    await loadDocument(docId);
    downloadButton.disabled = false;
    setStatus('Document loaded. Drag the overlays to transform objects.');
  } catch (error) {
    setStatus(`Failed to open document: ${error}`);
  } finally {
    input.value = '';
  }
});

downloadButton.addEventListener('click', () => {
  if (currentDocId) {
    api.download(currentDocId);
  }
});

async function loadDocument(docId: string): Promise<void> {
  const [ir, pdfBytes] = await Promise.all([api.getIR(docId), api.fetchPdf(docId)]);
  currentIR = ir;
  await preview.load(pdfBytes);
  buildPageStack();
  renderOverlay();
}

async function refreshIR(): Promise<void> {
  if (!currentDocId) {
    return;
  }
  currentIR = await api.getIR(currentDocId);
}

async function reloadPreviewFromDataUrl(dataUrl: string): Promise<void> {
  const base64 = dataUrl.split(',')[1] ?? dataUrl;
  const binary = atob(base64);
  const buffer = new ArrayBuffer(binary.length);
  const view = new Uint8Array(buffer);
  for (let i = 0; i < binary.length; i += 1) {
    view[i] = binary.charCodeAt(i);
  }
  await preview.load(buffer);
  buildPageStack();
}

function buildPageStack(): void {
  pageStack.innerHTML = '';
  const canvases = Array.from(pdfContainer.querySelectorAll('canvas'));
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
  });
}

function renderOverlay(): void {
  if (!currentIR) {
    return;
  }
  const overlayWrappers = Array.from(pageStack.querySelectorAll('.page-wrapper__overlay')) as HTMLElement[];
  const pageSizes: PageSize[] = Array.from(pageStack.querySelectorAll('canvas')).map((canvas) => ({
    width: (canvas as HTMLCanvasElement).width,
    height: (canvas as HTMLCanvasElement).height,
  }));
  overlayManager.mount(currentIR, overlayWrappers, pageSizes);
}

function setStatus(message: string): void {
  statusEl.textContent = message;
}

setStatus('Select a PDF to begin.');
