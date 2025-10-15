import './styles.css';

import type { DocumentIR, PatchOperation } from './types';
import { openDocument, loadIR, sendPatch, downloadPdf } from './api';
import { createPdfPreview } from './pdfPreview';
import { createOverlay, type FabricObjectMeta } from './fabricOverlay';
import { irObjectToFabric } from './mapping';
import type { Matrix } from './coords';

interface AppState {
  docId: string | null;
  ir: DocumentIR | null;
}

const state: AppState = {
  docId: null,
  ir: null,
};

const root = document.querySelector<HTMLDivElement>('#app');
if (!root) {
  throw new Error('Missing app container');
}

root.innerHTML = `
  <div class="layout">
    <header class="toolbar">
      <input type="file" id="file-input" accept="application/pdf" />
      <button id="download-btn" disabled>Download PDF</button>
      <span id="status"></span>
    </header>
    <main class="workspace">
      <aside class="sidebar">
        <h2>Pages</h2>
        <div id="preview"></div>
      </aside>
      <section class="editor">
        <div class="canvas-wrapper">
          <canvas id="overlay" width="800" height="1100"></canvas>
        </div>
      </section>
    </main>
  </div>
`;

const fileInput = root.querySelector<HTMLInputElement>('#file-input');
const downloadBtn = root.querySelector<HTMLButtonElement>('#download-btn');
const statusEl = root.querySelector<HTMLSpanElement>('#status');
const previewContainer = root.querySelector<HTMLDivElement>('#preview');
const overlayCanvas = root.querySelector<HTMLCanvasElement>('#overlay');

if (!fileInput || !downloadBtn || !statusEl || !previewContainer || !overlayCanvas) {
  throw new Error('Failed to bootstrap UI');
}

let overlayController = createOverlay(overlayCanvas, {
  onObjectTransform(meta, matrix) {
    handleObjectTransform(meta, matrix).catch((err) => setStatus(err.message));
  },
  onRequestEditText(meta) {
    setStatus(`Edit text not implemented (object ${meta.id})`);
  },
});

const previewHandlePromise = createPdfPreview(previewContainer);

fileInput.addEventListener('change', async () => {
  const file = fileInput.files?.[0];
  if (!file) return;
  try {
    setStatus('Uploading…');
    const { docId } = await openDocument(file);
    state.docId = docId;
    downloadBtn.disabled = false;
    setStatus('Fetching IR…');
    state.ir = await loadIR(docId);
    setStatus('Ready');
    const preview = await previewHandlePromise;
    preview.setDocument(state.ir);
    refreshOverlay();
  } catch (err) {
    console.error(err);
    setStatus(err instanceof Error ? err.message : 'Failed to open document');
  }
});

downloadBtn.addEventListener('click', async () => {
  if (!state.docId) return;
  try {
    setStatus('Downloading…');
    const blob = await downloadPdf(state.docId);
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = 'edited.pdf';
    a.click();
    URL.revokeObjectURL(url);
    setStatus('Download complete');
  } catch (err) {
    console.error(err);
    setStatus(err instanceof Error ? err.message : 'Download failed');
  }
});

function setStatus(message: string) {
  statusEl.textContent = message;
}

function refreshOverlay() {
  if (!state.ir) {
    overlayController.updateObjects([]);
    return;
  }
  const meta = state.ir.pages.flatMap((page) =>
    page.objects.map((obj) => irObjectToFabric(page, obj))
  );
  overlayController.updateObjects(
    meta.map((descriptor) => ({
      id: descriptor.id,
      kind: descriptor.kind,
      initialMatrix: descriptor.matrixPx,
    }))
  );
}

async function handleObjectTransform(meta: FabricObjectMeta, matrix: Matrix) {
  if (!state.docId || !state.ir) return;
  const operations: PatchOperation[] = [
    {
      op: 'transform',
      target: { page: 0, id: meta.id },
      deltaMatrixPt: matrix,
      kind: meta.kind,
    },
  ];
  try {
    setStatus('Sending patch…');
    const response = await sendPatch(state.docId, operations);
    if (!response.ok) {
      setStatus(response.error ?? 'Patch failed');
      return;
    }
    setStatus('Patch applied (dummy response)');
  } catch (err) {
    console.error(err);
    setStatus(err instanceof Error ? err.message : 'Patch failed');
  }
}

window.addEventListener('beforeunload', () => {
  overlayController.dispose();
});
