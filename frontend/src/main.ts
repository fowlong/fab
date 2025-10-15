import './styles.css';
import { PdfPreview } from './pdfPreview';
import { FabricOverlay } from './fabricOverlay';
import * as api from './api';
import type { DocumentIR, Matrix, PatchOperation } from './types';

const app = document.querySelector<HTMLDivElement>('#app');
if (!app) {
  throw new Error('App container missing');
}

app.innerHTML = `
  <div class="layout">
    <aside class="sidebar">
      <h1>PDF Transform MVP</h1>
      <p class="sidebar__intro">Upload a PDF, drag the overlays, and download the rewritten file.</p>
      <label class="button">
        <span>Select PDF…</span>
        <input id="file-input" type="file" accept="application/pdf" hidden />
      </label>
      <button id="download" class="button button--secondary" disabled>Download updated PDF</button>
      <div id="status" class="status">Ready.</div>
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

let currentDocId: string | null = null;
let currentPdfBytes: ArrayBuffer | null = null;
let overlay: FabricOverlay | null = null;

const pdfLayer = document.createElement('div');
pdfLayer.className = 'pdf-layer';
const preview = new PdfPreview(pdfLayer);

fileInput.addEventListener('change', async (event) => {
  const input = event.target as HTMLInputElement;
  if (!input.files || input.files.length === 0) {
    return;
  }
  const file = input.files[0];
  setStatus(`Uploading ${file.name}…`);
  try {
    const { docId } = await api.open(file);
    await loadDocument(docId, await api.getIR(docId), await api.download(docId));
    setStatus('Document ready. Drag the overlays to transform text and images.');
  } catch (err) {
    setStatus(`Failed to open document: ${String(err)}`);
  }
});

downloadButton.addEventListener('click', async () => {
  if (!currentDocId) {
    return;
  }
  try {
    const bytes = currentPdfBytes ?? (await api.download(currentDocId));
    const blob = new Blob([bytes], { type: 'application/pdf' });
    const url = URL.createObjectURL(blob);
    const anchor = document.createElement('a');
    anchor.href = url;
    anchor.download = `${currentDocId}.pdf`;
    anchor.click();
    URL.revokeObjectURL(url);
  } catch (err) {
    setStatus(`Download failed: ${String(err)}`);
  }
});

async function loadDocument(docId: string, ir: DocumentIR, pdfBytes: ArrayBuffer) {
  currentDocId = docId;
  currentPdfBytes = pdfBytes;
  downloadButton.disabled = false;

  overlay?.dispose();
  overlay = null;

  pageStack.innerHTML = '';
  const wrapper = document.createElement('div');
  wrapper.className = 'page-wrapper';
  const pdfContainer = document.createElement('div');
  pdfContainer.className = 'page-wrapper__pdf';
  const overlayContainer = document.createElement('div');
  overlayContainer.className = 'page-wrapper__overlay';
  const overlayCanvas = document.createElement('canvas');
  overlayCanvas.className = 'fabric-page-overlay';
  overlayContainer.appendChild(overlayCanvas);
  wrapper.appendChild(pdfContainer);
  wrapper.appendChild(overlayContainer);
  pageStack.appendChild(wrapper);

  pdfContainer.appendChild(pdfLayer);
  await preview.render(pdfBytes);
  const pdfCanvas = preview.getCanvas();
  if (!pdfCanvas) {
    throw new Error('Failed to render PDF page');
  }

  const sizePx = { width: pdfCanvas.width, height: pdfCanvas.height };
  overlayCanvas.width = sizePx.width;
  overlayCanvas.height = sizePx.height;

  const sizePt = preview.getSizePt();
  if (!sizePt) {
    throw new Error('Missing page size info');
  }

  overlay = new FabricOverlay(async (id, kind, delta) => {
    await handleTransform(id, kind, delta);
  }, sizePt.heightPt);
  overlay.mount(overlayCanvas);
  const page = ir.pages[0];
  overlay.sync(page.objects, sizePx);
}

async function handleTransform(id: string, kind: 'text' | 'image', delta: Matrix) {
  if (!currentDocId) {
    return;
  }
  setStatus('Applying transform…');
  const ops: PatchOperation[] = [
    {
      op: 'transform',
      target: { page: 0, id },
      deltaMatrixPt: delta,
      kind,
    },
  ];
  try {
    const response = await api.patch(currentDocId, ops);
    const nextPdf = response.updatedPdf
      ? dataUrlToArrayBuffer(response.updatedPdf)
      : await api.download(currentDocId);
    const ir = await api.getIR(currentDocId);
    await loadDocument(currentDocId, ir, nextPdf);
    setStatus('Transform applied.');
  } catch (err) {
    setStatus(`Transform failed: ${String(err)}`);
    throw err;
  }
}

function setStatus(message: string) {
  statusEl.textContent = message;
}

function dataUrlToArrayBuffer(dataUrl: string): ArrayBuffer {
  const [, raw] = dataUrl.split(',');
  const source = raw ?? dataUrl;
  const binary = atob(source);
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i += 1) {
    bytes[i] = binary.charCodeAt(i);
  }
  return bytes.buffer;
}
