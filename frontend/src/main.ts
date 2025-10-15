import './styles.css';
import type { DocumentIR, PageIR } from './types';
import { openDocument, getIR, sendPatch, downloadPdf } from './api';
import { FabricOverlay } from './fabricOverlay';

const app = document.getElementById('app');
if (!app) {
  throw new Error('App element missing');
}

const sidebar = document.createElement('aside');
sidebar.className = 'sidebar';
sidebar.innerHTML = `
  <h1>PDF Editor</h1>
  <p>Select a PDF to get started.</p>
  <input type="file" id="file-input" accept="application/pdf" />
  <div id="thumbnails"></div>
`;

const main = document.createElement('main');
main.className = 'main-panel';
main.innerHTML = `
  <div class="toolbar">
    <button id="download-btn">Download PDF</button>
  </div>
  <div class="canvas-stack" id="canvas-stack"></div>
`;

app.append(sidebar, main);

const fileInput = sidebar.querySelector<HTMLInputElement>('#file-input');
const stack = main.querySelector<HTMLDivElement>('#canvas-stack');
const downloadBtn = main.querySelector<HTMLButtonElement>('#download-btn');

let docId: string | undefined;
let ir: DocumentIR | undefined;
let overlays: FabricOverlay[] = [];
let currentPdfData: Uint8Array | undefined;

function clearOverlays() {
  overlays.forEach((overlay) => overlay.dispose());
  overlays = [];
  stack!.innerHTML = '';
}

function showStatus(message: string) {
  const toast = document.createElement('div');
  toast.className = 'status-toast';
  toast.textContent = message;
  document.body.appendChild(toast);
  setTimeout(() => toast.remove(), 2500);
}

async function handlePatches(ops: any[]) {
  if (!docId) return;
  try {
    const response = await sendPatch(docId, ops);
    if (response.updatedPdf) {
      const base64 = response.updatedPdf.split(',')[1] ?? response.updatedPdf;
      currentPdfData = Uint8Array.from(atob(base64), (c) => c.charCodeAt(0));
      showStatus('PDF updated');
    }
  } catch (error) {
    console.error(error);
    showStatus('Failed to apply patch');
  }
}

async function renderIRPages(irDoc: DocumentIR) {
  if (!stack) return;
  clearOverlays();
  irDoc.pages.forEach((page: PageIR) => {
    const wrapper = document.createElement('div');
    wrapper.className = 'canvas-wrapper';

    const underlay = document.createElement('canvas');
    underlay.width = page.widthPt;
    underlay.height = page.heightPt;
    wrapper.appendChild(underlay);

    const overlayCanvas = document.createElement('canvas');
    overlayCanvas.width = page.widthPt;
    overlayCanvas.height = page.heightPt;
    wrapper.appendChild(overlayCanvas);

    stack.appendChild(wrapper);

    const overlay = new FabricOverlay({
      canvas: overlayCanvas,
      page,
      onPatchRequested: handlePatches
    });
    overlays.push(overlay);
  });
}

async function bootstrap(file: File) {
  const open = await openDocument(file);
  docId = open.docId;
  ir = await getIR(docId);
  await renderIRPages(ir);
}

fileInput?.addEventListener('change', async (event) => {
  const files = (event.target as HTMLInputElement).files;
  if (!files || files.length === 0) {
    return;
  }
  const file = files[0];
  await bootstrap(file);
});

downloadBtn?.addEventListener('click', async () => {
  if (!docId) return;
  try {
    const blob = await downloadPdf(docId);
    const url = URL.createObjectURL(blob);
    const anchor = document.createElement('a');
    anchor.href = url;
    anchor.download = 'edited.pdf';
    anchor.click();
    URL.revokeObjectURL(url);
  } catch (error) {
    console.error(error);
    showStatus('Download failed');
  }
});
