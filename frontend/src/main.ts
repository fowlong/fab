import './styles.css';
import { FabricOverlay } from './fabricOverlay';
import { PdfPreview } from './pdfPreview';
import { download, fetchPdf, getIR, open, patch } from './api';
import type { DocumentIR, PageIR } from './types';

import * as FabricNS from 'fabric';
const fabric: typeof import('fabric')['fabric'] = (FabricNS as any);

const app = document.querySelector<HTMLDivElement>('#app');
if (!app) {
  throw new Error('App container missing');
}

app.innerHTML = `
  <div class="layout">
    <aside class="sidebar">
      <h1>PDF Transform MVP</h1>
      <p>Upload a single-page PDF containing text and an image, then drag the overlay controllers to rewrite the PDF.</p>
      <label class="button button--secondary">
        <span>Select PDF…</span>
        <input id="file-input" type="file" accept="application/pdf" hidden />
      </label>
      <button id="download-btn" class="button" disabled>Download updated PDF</button>
      <div id="status" class="status"></div>
    </aside>
    <section class="editor">
      <div id="page-wrapper" class="page-stack"></div>
    </section>
  </div>
`;

const statusEl = document.getElementById('status') as HTMLDivElement;
const fileInput = document.getElementById('file-input') as HTMLInputElement;
const downloadBtn = document.getElementById('download-btn') as HTMLButtonElement;
const pageWrapper = document.getElementById('page-wrapper') as HTMLDivElement;

const previewContainer = document.createElement('div');
previewContainer.className = 'page-wrapper pdf-container';
pageWrapper.appendChild(previewContainer);

const preview = new PdfPreview(previewContainer);
let overlay: FabricOverlay | null = null;
let fabricCanvas: fabric.Canvas | null = null;
let currentDocId: string | null = null;

fileInput.addEventListener('change', async (event) => {
  const input = event.currentTarget as HTMLInputElement;
  if (!input.files || input.files.length === 0) {
    return;
  }
  const file = input.files[0];
  input.value = '';
  try {
    await loadDocument(file);
  } catch (err) {
    console.error(err);
    setStatus(`Failed to load document: ${String((err as Error).message ?? err)}`);
  }
});

downloadBtn.addEventListener('click', () => {
  if (!currentDocId) {
    setStatus('Upload a document before downloading.');
    return;
  }
  download(currentDocId).catch((err) => {
    setStatus(`Download failed: ${err instanceof Error ? err.message : String(err)}`);
  });
});

async function loadDocument(file: File) {
  setStatus(`Uploading “${file.name}”…`);
  const { docId } = await open(file);
  currentDocId = docId;
  downloadBtn.disabled = false;
  setStatus('Parsing IR…');
  const ir = await getIR(docId);
  await renderDocument(ir);
  setStatus('Ready. Drag the controllers to transform the PDF.');
}

async function renderDocument(ir: DocumentIR) {
  if (!currentDocId) return;
  if (ir.pages.length === 0) {
    throw new Error('No pages in IR');
  }
  const page = ir.pages[0];
  const buffer = await fetchPdf(currentDocId);
  const metrics = await preview.render(buffer);
  setupOverlay(metrics.canvas, metrics.heightPt, page, currentDocId);
}

function setupOverlay(canvasEl: HTMLCanvasElement, pageHeightPt: number, page: PageIR, docId: string) {
  if (fabricCanvas) {
    fabricCanvas.dispose();
    fabricCanvas = null;
    overlay = null;
  }
  const overlayCanvasEl = document.createElement('canvas');
  overlayCanvasEl.width = canvasEl.width;
  overlayCanvasEl.height = canvasEl.height;
  overlayCanvasEl.className = 'fabric-overlay';

  const wrapper = canvasEl.parentElement as HTMLElement;
  wrapper.innerHTML = '';
  const underlayHolder = document.createElement('div');
  underlayHolder.className = 'page-wrapper__pdf';
  underlayHolder.appendChild(canvasEl);
  const overlayHolder = document.createElement('div');
  overlayHolder.className = 'page-wrapper__overlay';
  overlayHolder.appendChild(overlayCanvasEl);
  wrapper.appendChild(underlayHolder);
  wrapper.appendChild(overlayHolder);

  fabricCanvas = new fabric.Canvas(overlayCanvasEl, {
    selection: false,
    preserveObjectStacking: true,
  });

  overlay = new FabricOverlay(fabricCanvas, pageHeightPt, async ({ id, page: pageIndex, kind, delta }) => {
    if (!currentDocId) return;
    const patchPayload = [{
      op: 'transform' as const,
      target: { page: pageIndex, id },
      deltaMatrixPt: delta,
      kind,
    }];
    try {
      const response = await patch(docId, patchPayload);
      if (response.updatedPdf) {
        const buffer = dataUrlToArrayBuffer(response.updatedPdf);
        await preview.render(buffer);
        const refreshed = await getIR(docId);
        overlay?.setObjects(refreshed.pages[0]);
      }
      setStatus('Transform applied.');
    } catch (err) {
      console.error(err);
      setStatus(`Patch failed: ${err instanceof Error ? err.message : String(err)}`);
    }
  });

  overlay.setObjects(page);
}

function setStatus(message: string) {
  statusEl.textContent = message;
}

function dataUrlToArrayBuffer(dataUrl: string): ArrayBuffer {
  const [, base64] = dataUrl.split(',', 2);
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

setStatus('Select a PDF to begin.');
