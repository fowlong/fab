import './styles.css';
import { open, getIR, patch, download } from './api';
import type { DocumentIR, PatchOperation } from './types';
import { PdfPreview } from './pdfPreview';
import { FabricOverlay, type TransformHandler } from './fabricOverlay';

function createLayout() {
  const app = document.querySelector<HTMLDivElement>('#app');
  if (!app) {
    throw new Error('App container missing');
  }
  app.innerHTML = `
    <div class="layout">
      <header class="toolbar">
        <label class="file-input">
          <input type="file" id="file-picker" accept="application/pdf" />
          <span>Select PDF…</span>
        </label>
        <button id="download-btn" class="button" disabled>Download updated PDF</button>
        <span id="status" class="status">Idle.</span>
      </header>
      <main class="viewer">
        <div id="page-wrapper" class="page-wrapper"></div>
      </main>
    </div>
  `;
}

function dataUriToArrayBuffer(uri: string): ArrayBuffer {
  const [, base64] = uri.split(',', 2);
  const binary = atob(base64 ?? '');
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i += 1) {
    bytes[i] = binary.charCodeAt(i);
  }
  return bytes.buffer;
}

createLayout();

const statusEl = document.getElementById('status') as HTMLSpanElement;
const pageWrapper = document.getElementById('page-wrapper') as HTMLDivElement;
const filePicker = document.getElementById('file-picker') as HTMLInputElement;
const downloadBtn = document.getElementById('download-btn') as HTMLButtonElement;

const preview = new PdfPreview(pageWrapper);
let overlay: FabricOverlay | null = null;
let currentDoc: { id: string; ir: DocumentIR | null } = { id: '', ir: null };

function setStatus(message: string) {
  statusEl.textContent = message;
}

async function renderPdf(buffer: ArrayBuffer) {
  const result = await preview.render(buffer);
  const pdfCanvas = result.canvas;
  if (pdfCanvas.parentElement === pageWrapper) {
    pageWrapper.removeChild(pdfCanvas);
  }
  const wrapper = document.createElement('div');
  wrapper.className = 'page-wrapper';
  wrapper.style.width = `${result.widthPx}px`;
  wrapper.style.height = `${result.heightPx}px`;
  wrapper.appendChild(pdfCanvas);
  pageWrapper.appendChild(wrapper);

  const overlayCanvas = document.createElement('canvas');
  overlayCanvas.width = result.widthPx;
  overlayCanvas.height = result.heightPx;
  overlayCanvas.className = 'fabric-overlay';
  wrapper.appendChild(overlayCanvas);
  return overlayCanvas;
}

async function refresh(docId: string, ir: DocumentIR, updatedPdf?: string) {
  preview.reset();
  pageWrapper.innerHTML = '';
  const page = ir.pages[0];
  let buffer: ArrayBuffer;
  if (updatedPdf) {
    buffer = dataUriToArrayBuffer(updatedPdf);
  } else {
    const blob = await download(docId);
    buffer = await blob.arrayBuffer();
  }
  const overlayCanvas = await renderPdf(buffer);
  overlay = new FabricOverlay(overlayCanvas, page.heightPt, createHandler(docId));
  overlay.render(page);
}

function createHandler(docId: string): TransformHandler {
  return async ({ id, kind, delta }) => {
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
      const response = await patch(docId, ops);
      const ir = await getIR(docId);
      currentDoc.ir = ir;
      await refresh(docId, ir, response.updatedPdf);
      setStatus('Transform applied.');
    } catch (err) {
      setStatus(`Transform failed: ${err}`);
      throw err;
    }
  };
}

filePicker.addEventListener('change', async (event) => {
  const input = event.target as HTMLInputElement;
  if (!input.files || input.files.length === 0) {
    return;
  }
  const file = input.files[0];
  try {
    setStatus('Uploading…');
    downloadBtn.disabled = true;
    const { docId } = await open(file);
    currentDoc = { id: docId, ir: null };
    const ir = await getIR(docId);
    currentDoc.ir = ir;
    await refresh(docId, ir);
    downloadBtn.disabled = false;
    setStatus('Ready. Drag controllers to transform.');
  } catch (err) {
    console.error(err);
    setStatus(`Failed to load PDF: ${err}`);
  }
});

downloadBtn.addEventListener('click', async () => {
  if (!currentDoc.id) return;
  try {
    setStatus('Preparing download…');
    const blob = await download(currentDoc.id);
    const url = URL.createObjectURL(blob);
    const anchor = document.createElement('a');
    anchor.href = url;
    anchor.download = `${currentDoc.id}.pdf`;
    anchor.click();
    URL.revokeObjectURL(url);
    setStatus('Download ready.');
  } catch (err) {
    console.error(err);
    setStatus(`Download failed: ${err}`);
  }
});

setStatus('Select a PDF to begin.');
