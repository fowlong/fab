import './styles.css';
import { PdfPreview } from './pdfPreview';
import { FabricOverlayManager } from './fabricOverlay';
import { downloadPdf, fetchIR, fetchPdfBytes, openDocument } from './api';
import type { DocumentIR } from './types';

const app = document.querySelector<HTMLDivElement>('#app');
if (!app) {
  throw new Error('App container missing');
}

app.innerHTML = `
  <div class="layout">
    <aside class="sidebar">
      <h1>PDF Transformer</h1>
      <label class="button">
        <span>Select PDF…</span>
        <input id="file-input" type="file" accept="application/pdf" hidden />
      </label>
      <button id="download" class="button button--secondary" disabled>Download updated PDF</button>
      <div id="status" class="status"></div>
    </aside>
    <section class="editor">
      <div id="page-wrapper" class="page-wrapper">
        <div id="pdf-underlay" class="page-underlay"></div>
        <div id="overlay-host" class="page-overlay"></div>
      </div>
    </section>
  </div>
`;

const statusEl = document.getElementById('status') as HTMLDivElement;
const fileInput = document.getElementById('file-input') as HTMLInputElement;
const downloadBtn = document.getElementById('download') as HTMLButtonElement;
const underlay = document.getElementById('pdf-underlay') as HTMLDivElement;
const overlayHost = document.getElementById('overlay-host') as HTMLDivElement;

const preview = new PdfPreview(underlay);
const overlayManager = new FabricOverlayManager();
let currentDocId: string | null = null;

fileInput.addEventListener('change', async (event) => {
  const input = event.target as HTMLInputElement;
  if (!input.files || input.files.length === 0) {
    return;
  }
  const file = input.files[0];
  setStatus(`Uploading ${file.name}…`);
  try {
    const { docId } = await openDocument(file);
    currentDocId = docId;
    downloadBtn.disabled = false;
    await loadDocument(docId);
    setStatus(`Loaded ${file.name}. Drag, rotate, or scale the overlays.`);
  } catch (err) {
    console.error(err);
    setStatus('Failed to load document.');
  }
});

downloadBtn.addEventListener('click', () => {
  if (!currentDocId) return;
  void downloadPdf(currentDocId).catch((err) => {
    console.error(err);
    setStatus('Download failed.');
  });
});

async function loadDocument(docId: string) {
  const [ir, pdfBytes] = await Promise.all([fetchIR(docId), fetchPdfBytes(docId)]);
  await preview.render(pdfBytes);
  mountOverlay(docId, ir);
}

function mountOverlay(docId: string, ir: DocumentIR) {
  const canvas = preview.getCanvas();
  const pageSize = preview.getPageSize();
  if (!canvas || !pageSize) {
    throw new Error('Preview canvas missing');
  }
  const page = ir.pages[0];
  void overlayManager.mount({
    docId,
    ir,
    overlayHost,
    pageCanvasSize: { widthPx: canvas.width, heightPx: canvas.height },
    pageSizePt: { widthPt: page.widthPt, heightPt: page.heightPt },
    onPatched: async () => {
      if (!currentDocId) return;
      const [nextIr, pdfBytes] = await Promise.all([
        fetchIR(currentDocId),
        fetchPdfBytes(currentDocId),
      ]);
      await preview.render(pdfBytes);
      mountOverlay(currentDocId, nextIr);
    },
  });
}

function setStatus(message: string) {
  statusEl.textContent = message;
}

setStatus('Select a PDF to begin editing.');
