import { fabric } from 'fabric';
import { initFabricCanvas, buildOverlay, wireTextEditing, type OverlayContext } from './fabricOverlay';
import { fetchIR, openDocument, sendPatch, downloadPdf } from './api';
import type { DocumentIR } from './types';

const app = document.getElementById('app');
if (!app) {
  throw new Error('App container missing');
}

app.innerHTML = `
  <div class="app-shell">
    <aside class="sidebar">
      <h1>PDF Editor</h1>
      <input type="file" accept="application/pdf" id="pdf-file" />
      <button id="download-btn" disabled>Download PDF</button>
      <div id="status"></div>
    </aside>
    <main class="editor">
      <div class="page-container">
        <canvas id="pdf-canvas"></canvas>
        <canvas id="fabric-canvas"></canvas>
      </div>
    </main>
  </div>
`;

const pdfCanvas = document.getElementById('pdf-canvas') as HTMLCanvasElement;
const fabricCanvasEl = document.getElementById('fabric-canvas') as HTMLCanvasElement;
const fileInput = document.getElementById('pdf-file') as HTMLInputElement;
const downloadBtn = document.getElementById('download-btn') as HTMLButtonElement;
const statusEl = document.getElementById('status') as HTMLDivElement;

const fabricCanvas = initFabricCanvas(fabricCanvasEl);

let currentDocId: string | null = null;
let currentIR: DocumentIR | null = null;

async function handleFileChange(event: Event) {
  const input = event.target as HTMLInputElement;
  if (!input.files || input.files.length === 0) {
    return;
  }
  const file = input.files[0];
  statusEl.textContent = 'Uploading…';
  try {
    const docId = await openDocument(file);
    currentDocId = docId;
    downloadBtn.disabled = false;
    statusEl.textContent = 'Loading IR…';
    currentIR = await fetchIR(docId);
    statusEl.textContent = 'Rendering page 1…';
    if (currentIR.pages.length === 0) {
      statusEl.textContent = 'No pages found in PDF.';
      return;
    }
    const page = currentIR.pages[0];
    await renderPreview(`/api/pdf/${docId}`);
    const ctx: OverlayContext = {
      canvas: fabricCanvas,
      bindings: new Map(),
      pageHeightPt: page.heightPt,
      docId,
      pageIndex: 0,
      onPatch: async (ops) => {
        if (!currentDocId) return;
        statusEl.textContent = 'Applying patch…';
        await sendPatch(currentDocId, ops);
        statusEl.textContent = 'Patch applied.';
      }
    };
    buildOverlay(ctx, page.objects);
    wireTextEditing(ctx, currentIR, app);
    statusEl.textContent = 'Ready.';
  } catch (err) {
    console.error(err);
    statusEl.textContent = err instanceof Error ? err.message : String(err);
  }
}

async function renderPreview(url: string) {
  const pdfjsLib = await import('pdfjs-dist/build/pdf');
  pdfjsLib.GlobalWorkerOptions.workerSrc = `https://cdnjs.cloudflare.com/ajax/libs/pdf.js/${pdfjsLib.version}/pdf.worker.min.js`;
  const pdf = await pdfjsLib.getDocument(url).promise;
  const page = await pdf.getPage(1);
  const viewport = page.getViewport({ scale: 1.0 });
  const context = pdfCanvas.getContext('2d');
  if (!context) {
    throw new Error('Failed to acquire canvas context');
  }
  pdfCanvas.width = viewport.width;
  pdfCanvas.height = viewport.height;
  fabricCanvas.setWidth(viewport.width);
  fabricCanvas.setHeight(viewport.height);
  await page.render({ canvasContext: context, viewport }).promise;
}

async function handleDownload() {
  if (!currentDocId) {
    return;
  }
  const blob = await downloadPdf(currentDocId);
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = 'edited.pdf';
  a.click();
  URL.revokeObjectURL(url);
}

fileInput.addEventListener('change', handleFileChange);
downloadBtn.addEventListener('click', handleDownload);

fabricCanvas.on('selection:created', (evt) => {
  if (evt.selected?.length) {
    statusEl.textContent = `Selected ${evt.selected[0].type}`;
  }
});

fabricCanvas.on('selection:cleared', () => {
  statusEl.textContent = 'Ready.';
});
