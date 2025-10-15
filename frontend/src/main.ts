import './styles.css';

import { PdfPreview } from './pdfPreview';
import { FabricOverlayManager } from './fabricOverlay';
import { fabricDeltaToPdfDelta } from './coords';
import {
  downloadPdf,
  downloadToFile,
  getIR,
  openDocument,
  openSample,
  patchDocument,
} from './api';
import type { DocumentIR, PatchOperation } from './types';

const app = document.querySelector<HTMLDivElement>('#app');
if (!app) {
  throw new Error('App container missing');
}

app.innerHTML = `
  <div class="layout">
    <aside class="sidebar">
      <h1>PDF Transform MVP</h1>
      <p class="sidebar__intro">Upload a PDF or load the bundled sample, then drag the overlay controllers to transform text runs and images.</p>
      <div class="sidebar__controls">
        <button id="load-sample" class="button">Load sample</button>
        <label class="button button--secondary">
          <span>Select PDF…</span>
          <input id="file-input" type="file" accept="application/pdf" hidden />
        </label>
        <button id="download" class="button button--secondary" disabled>Download PDF</button>
      </div>
      <div id="status" class="status">Ready.</div>
    </aside>
    <section class="editor">
      <div id="page-stack" class="page-stack"></div>
    </section>
  </div>
`;

const statusEl = document.getElementById('status') as HTMLDivElement;
const pageStack = document.getElementById('page-stack') as HTMLDivElement;
const loadSampleButton = document.getElementById('load-sample') as HTMLButtonElement;
const fileInput = document.getElementById('file-input') as HTMLInputElement;
const downloadButton = document.getElementById('download') as HTMLButtonElement;

const pdfContainer = document.createElement('div');
const preview = new PdfPreview(pdfContainer);

let currentDocId: string | null = null;

const overlayManager = new FabricOverlayManager(async ({ id, kind, fold, next, pageHeightPt }) => {
  if (!currentDocId) {
    return false;
  }
  const patch: PatchOperation = {
    op: 'transform',
    target: { page: 0, id },
    deltaMatrixPt: fabricDeltaToPdfDelta(fold, next, pageHeightPt),
    kind,
  };

  try {
    setStatus('Applying transform…');
    const response = await patchDocument(currentDocId, [patch]);
    if (response.updatedPdf) {
      const buffer = await dataUrlToArrayBuffer(response.updatedPdf);
      await refreshDocument(currentDocId, buffer);
    } else {
      await refreshDocument(currentDocId);
    }
    setStatus('Transform applied.');
    return true;
  } catch (error) {
    console.error(error);
    setStatus(`Transform failed: ${(error as Error).message}`);
    return false;
  }
});

loadSampleButton.addEventListener('click', () => {
  handleOpenSample().catch((error) => {
    console.error(error);
    setStatus(`Failed to load sample: ${(error as Error).message}`);
  });
});

fileInput.addEventListener('change', () => {
  const file = fileInput.files?.[0];
  if (!file) {
    return;
  }
  handleOpenFile(file).catch((error) => {
    console.error(error);
    setStatus(`Failed to open file: ${(error as Error).message}`);
  });
  fileInput.value = '';
});

downloadButton.addEventListener('click', () => {
  if (!currentDocId) {
    return;
  }
  setStatus('Preparing download…');
  downloadPdf(currentDocId)
    .then((blob) => {
      downloadToFile(blob, `${currentDocId}.pdf`);
      setStatus('Download ready.');
    })
    .catch((error) => {
      console.error(error);
      setStatus(`Download failed: ${(error as Error).message}`);
    });
});

async function handleOpenSample(): Promise<void> {
  setStatus('Opening sample…');
  const response = await openSample();
  currentDocId = response.docId;
  await refreshDocument(response.docId);
  downloadButton.disabled = false;
  setStatus('Sample loaded.');
}

async function handleOpenFile(file: File): Promise<void> {
  setStatus(`Uploading ${file.name}…`);
  const response = await openDocument(file);
  currentDocId = response.docId;
  await refreshDocument(response.docId);
  downloadButton.disabled = false;
  setStatus(`${file.name} loaded.`);
}

async function refreshDocument(docId: string, pdfBuffer?: ArrayBuffer): Promise<void> {
  const [ir, buffer] = await Promise.all([
    getIR(docId),
    pdfBuffer ? Promise.resolve(pdfBuffer) : fetchPdfBuffer(docId),
  ]);
  await renderDocument(ir, buffer);
}

async function fetchPdfBuffer(docId: string): Promise<ArrayBuffer> {
  const blob = await downloadPdf(docId);
  return blob.arrayBuffer();
}

async function renderDocument(ir: DocumentIR, pdfBuffer: ArrayBuffer): Promise<void> {
  pageStack.innerHTML = '';
  pdfContainer.innerHTML = '';
  await preview.load(pdfBuffer);
  const canvases = preview.getCanvases();
  const sizes = preview.getSizes();
  const overlayWrappers: HTMLElement[] = [];

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
    overlayWrappers[index] = overlayLayer;
  });

  overlayManager.populate(ir, overlayWrappers, sizes);
}

function setStatus(message: string): void {
  statusEl.textContent = message;
}

async function dataUrlToArrayBuffer(url: string): Promise<ArrayBuffer> {
  const response = await fetch(url);
  if (!response.ok) {
    throw new Error(`Failed to load updated PDF: ${response.status}`);
  }
  return response.arrayBuffer();
}

setStatus('Ready. Load a PDF to begin.');
