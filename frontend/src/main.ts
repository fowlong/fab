import './styles.css';
import { PdfPreview } from './pdfPreview';
import { FabricOverlayManager } from './fabricOverlay';
import * as api from './api';
import type { DocumentIR } from './types';

const app = document.querySelector<HTMLDivElement>('#app');
if (!app) {
  throw new Error('App container missing');
}

app.innerHTML = `
  <div class="layout">
    <aside class="sidebar">
      <h1>PDF Transform MVP</h1>
      <p class="sidebar__intro">Upload a PDF, drag the overlays, and download an incrementally updated document.</p>
      <label class="button">
        <span>Select PDF…</span>
        <input id="file-input" type="file" accept="application/pdf" hidden />
      </label>
      <button id="download" class="button button--secondary" disabled>Download updated PDF</button>
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

const overlayManager = new FabricOverlayManager(async (id, kind, delta) => {
  if (!currentDocId) {
    return;
  }
  setStatus('Applying transform…');
  try {
    const response = await api.patch(currentDocId, [
      {
        op: 'transform',
        target: { page: 0, id },
        deltaMatrixPt: delta,
        kind,
      },
    ]);
    const ir = await api.getIR(currentDocId);
    const pdfBuffer =
      response.updatedPdf !== undefined
        ? decodeDataUri(response.updatedPdf)
        : await downloadPdfBuffer(currentDocId);
    await render(ir, pdfBuffer);
    setStatus('Transform applied.');
  } catch (error) {
    console.error(error);
    if (currentDocId) {
      await reloadDocument(currentDocId).catch((reloadErr) => {
        console.error('Failed to reload document after error', reloadErr);
      });
    }
    setStatus(`Transform failed: ${String(error)}`);
  }
});

fileInput.addEventListener('change', async (event) => {
  const input = event.target as HTMLInputElement;
  if (!input.files || input.files.length === 0) {
    return;
  }
  const file = input.files[0];
  setStatus(`Uploading ${file.name}…`);
  try {
    const { docId } = await api.open(file);
    currentDocId = docId;
    downloadButton.disabled = false;
    await reloadDocument(docId);
    setStatus('Document loaded. Drag the overlays to transform the PDF.');
  } catch (error) {
    console.error(error);
    setStatus(`Failed to open document: ${String(error)}`);
  } finally {
    input.value = '';
  }
});

downloadButton.addEventListener('click', async () => {
  if (!currentDocId) {
    return;
  }
  try {
    const blob = await api.download(currentDocId);
    const url = URL.createObjectURL(blob);
    const anchor = document.createElement('a');
    anchor.href = url;
    anchor.download = `${currentDocId}.pdf`;
    document.body.appendChild(anchor);
    anchor.click();
    anchor.remove();
    URL.revokeObjectURL(url);
  } catch (error) {
    console.error(error);
    setStatus(`Download failed: ${String(error)}`);
  }
});

async function reloadDocument(docId: string) {
  const ir = await api.getIR(docId);
  const pdfBuffer = await downloadPdfBuffer(docId);
  await render(ir, pdfBuffer);
}

async function render(ir: DocumentIR, pdfBuffer: ArrayBuffer) {
  pageStack.innerHTML = '';
  pdfContainer.innerHTML = '';
  await preview.load(pdfBuffer);
  const sizes = preview.getSizes();
  const canvases = Array.from(pdfContainer.querySelectorAll('canvas'));
  await overlayManager.reset();

  for (let index = 0; index < canvases.length; index += 1) {
    const canvas = canvases[index];
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

    const page = ir.pages[index];
    if (page && sizes[index]) {
      await overlayManager.render(page, overlayLayer, sizes[index]);
    }
  }
}

async function downloadPdfBuffer(docId: string): Promise<ArrayBuffer> {
  const blob = await api.download(docId);
  return blob.arrayBuffer();
}

function decodeDataUri(dataUri: string): ArrayBuffer {
  const [, base64] = dataUri.split(',');
  if (!base64) {
    throw new Error('Invalid data URI');
  }
  const binary = atob(base64);
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i += 1) {
    bytes[i] = binary.charCodeAt(i);
  }
  return bytes.buffer;
}

function setStatus(message: string) {
  statusEl.textContent = message;
}

setStatus('Select a PDF to begin.');
