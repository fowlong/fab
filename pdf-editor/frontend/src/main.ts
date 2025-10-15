import './styles.css';
import { openDocument, fetchIr, downloadPdf, sendPatchOperations } from './api';
import { renderPage } from './pdfPreview';
import { setupOverlay } from './fabricOverlay';
import type { DocumentIr, PatchOperation } from './types';

const app = document.getElementById('app');
if (!app) {
  throw new Error('Missing root element');
}

const state: {
  docId: string | null;
  ir: DocumentIr | null;
  pdfBytes: Uint8Array | null;
} = {
  docId: null,
  ir: null,
  pdfBytes: null,
};

const toolbar = document.createElement('div');
toolbar.className = 'toolbar';

const openButton = document.createElement('button');
openButton.textContent = 'Open sample PDF';
openButton.addEventListener('click', async () => {
  openButton.disabled = true;
  try {
    const docId = await openDocument('sample.pdf');
    state.docId = docId;
    state.ir = await fetchIr(docId);
    const blob = await downloadPdf(docId);
    const arrayBuffer = await blob.arrayBuffer();
    state.pdfBytes = new Uint8Array(arrayBuffer);
    renderWorkspace();
  } catch (err) {
    console.error(err);
    alert('Failed to open document');
  } finally {
    openButton.disabled = false;
  }
});

toolbar.appendChild(openButton);

const downloadButton = document.createElement('button');
downloadButton.textContent = 'Download current PDF';
downloadButton.addEventListener('click', async () => {
  if (!state.docId) {
    return;
  }
  const blob = await downloadPdf(state.docId);
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = 'edited.pdf';
  a.click();
  URL.revokeObjectURL(url);
});
toolbar.appendChild(downloadButton);

const workspace = document.createElement('div');
workspace.className = 'workspace';

app.appendChild(toolbar);
app.appendChild(workspace);

function renderWorkspace() {
  workspace.innerHTML = '';
  if (!state.ir || !state.pdfBytes || !state.docId) {
    const placeholder = document.createElement('p');
    placeholder.textContent = 'Open a PDF to begin editing.';
    workspace.appendChild(placeholder);
    return;
  }

  state.ir.pages.forEach((page) => {
    const pageContainer = document.createElement('div');
    pageContainer.className = 'page-container';

    const previewCanvas = document.createElement('canvas');
    previewCanvas.className = 'pdf-preview';

    const overlayCanvas = document.createElement('canvas');
    overlayCanvas.className = 'fabric-overlay';

    const canvasWrap = document.createElement('div');
    canvasWrap.className = 'canvas-wrap';
    canvasWrap.append(previewCanvas, overlayCanvas);

    pageContainer.appendChild(canvasWrap);
    workspace.appendChild(pageContainer);

    renderPage(state.pdfBytes!, previewCanvas, page.index).then(() => {
      overlayCanvas.width = previewCanvas.width;
      overlayCanvas.height = previewCanvas.height;
      setupOverlay({
        canvas: overlayCanvas,
        page,
        onPatch: async (ops: PatchOperation[]) => {
          if (!state.docId) {
            return;
          }
          await sendPatchOperations(state.docId, ops);
        },
      });
    });
  });
}

renderWorkspace();
