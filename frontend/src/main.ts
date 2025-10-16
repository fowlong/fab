import './styles.css';
import { PdfPreview } from './pdfPreview';
import { FabricOverlayManager } from './fabricOverlay';
import { open, getIR, patch, fetchPdfBytes, download } from './api';
import type { DocumentIR, PatchOperation } from './types';
import type { Matrix } from './coords';

const app = document.querySelector<HTMLDivElement>('#app');
if (!app) {
  throw new Error('App container missing');
}

type LoadedDoc = {
  id: string;
  ir: DocumentIR;
  buffer: ArrayBuffer;
};

let currentDoc: LoadedDoc | null = null;
let isPatching = false;

app.innerHTML = `
  <div class="layout">
    <aside class="sidebar">
      <h1>PDF Transform MVP</h1>
      <p class="sidebar__intro">Upload a PDF to manipulate text and image placement with Fabric.js overlays.</p>
      <label class="button">
        <span>Select PDF…</span>
        <input id="file-input" type="file" accept="application/pdf" />
      </label>
      <button id="download-btn" class="button button--secondary" disabled>Download current PDF</button>
      <div id="status" class="status"></div>
    </aside>
    <section class="editor">
      <div id="page-stack" class="page-stack"></div>
    </section>
  </div>
`;

const statusEl = document.getElementById('status') as HTMLDivElement;
const fileInput = document.getElementById('file-input') as HTMLInputElement;
const downloadBtn = document.getElementById('download-btn') as HTMLButtonElement;
const pageStack = document.getElementById('page-stack') as HTMLDivElement;

const wrapper = document.createElement('div');
wrapper.className = 'page-wrapper';
const pdfLayer = document.createElement('div');
pdfLayer.className = 'page-wrapper__pdf';
const overlayLayer = document.createElement('div');
overlayLayer.className = 'page-wrapper__overlay';
wrapper.appendChild(pdfLayer);
wrapper.appendChild(overlayLayer);
pageStack.appendChild(wrapper);

const preview = new PdfPreview(pdfLayer);
const overlay = new FabricOverlayManager();

fileInput.addEventListener('change', async (event) => {
  const input = event.target as HTMLInputElement;
  if (!input.files || input.files.length === 0) {
    return;
  }
  try {
    await loadDocument(input.files[0]);
  } catch (error) {
    console.error(error);
    setStatus(`Failed to load document: ${(error as Error).message}`);
  } finally {
    input.value = '';
  }
});

downloadBtn.addEventListener('click', () => {
  if (!currentDoc) {
    return;
  }
  download(currentDoc.id).catch((error) => {
    console.error(error);
    setStatus('Download failed.');
  });
});

async function loadDocument(file: File) {
  setStatus('Uploading PDF…');
  const { docId } = await open(file);
  const [ir, buffer] = await Promise.all([getIR(docId), fetchPdfBytes(docId)]);
  await renderDocument({ id: docId, ir, buffer });
  setStatus('Document ready. Drag the overlays to transform.');
  downloadBtn.disabled = false;
}

async function renderDocument(doc: LoadedDoc) {
  currentDoc = doc;
  await preview.render(doc.buffer);
  const size = preview.getSizePx();
  if (!size) {
    throw new Error('Preview size unavailable');
  }
  if (doc.ir.pages.length === 0) {
    throw new Error('IR missing page data');
  }
  const page = doc.ir.pages[0];
  overlay.mount(overlayLayer, size, page.heightPt, handleTransform);
  await overlay.populate(page);
}

async function handleTransform(payload: { id: string; kind: 'text' | 'image'; pageIndex: number; delta: Matrix }) {
  if (!currentDoc) {
    throw new Error('No document loaded');
  }
  if (isPatching) {
    throw new Error('Another transform is in progress');
  }
  isPatching = true;
  try {
    setStatus('Applying transform…');
    const op: PatchOperation = {
      op: 'transform',
      target: { page: payload.pageIndex, id: payload.id },
      deltaMatrixPt: payload.delta,
      kind: payload.kind,
    };
    const response = await patch(currentDoc.id, [op]);
    const buffer = response.updatedPdf
      ? dataUrlToArrayBuffer(response.updatedPdf)
      : await fetchPdfBytes(currentDoc.id);
    const ir = await getIR(currentDoc.id);
    await renderDocument({ id: currentDoc.id, ir, buffer });
    setStatus('Transform applied.');
  } catch (error) {
    setStatus(`Transform failed: ${(error as Error).message}`);
    throw error;
  } finally {
    isPatching = false;
  }
}

function dataUrlToArrayBuffer(dataUrl: string): ArrayBuffer {
  const [, base64 = ''] = dataUrl.split(',');
  const binary = atob(base64);
  const len = binary.length;
  const buffer = new Uint8Array(len);
  for (let i = 0; i < len; i += 1) {
    buffer[i] = binary.charCodeAt(i);
  }
  return buffer.buffer;
}

function setStatus(message: string) {
  statusEl.textContent = message;
}

setStatus('Select a PDF to begin.');
