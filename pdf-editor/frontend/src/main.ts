import './styles.css';
import { openDocument, fetchIr, applyPatch, downloadPdf } from './api';
import { renderPdf } from './pdfPreview';
import { setupOverlay, type OverlayHandle } from './fabricOverlay';
import type { DocumentIr, PatchOp } from './types';

const app = document.querySelector<HTMLDivElement>('#app');
if (!app) {
  throw new Error('Missing app root');
}

app.innerHTML = `
  <div class="layout">
    <aside class="sidebar">
      <h1>PDF Editor</h1>
      <label class="file-input">Open PDF<input type="file" id="fileInput" accept="application/pdf" /></label>
      <button id="downloadBtn" disabled>Download PDF</button>
      <div id="status" class="status">No document loaded.</div>
    </aside>
    <main class="editor" id="editor"></main>
  </div>
`;

const fileInput = document.getElementById('fileInput') as HTMLInputElement;
const downloadBtn = document.getElementById('downloadBtn') as HTMLButtonElement;
const statusEl = document.getElementById('status') as HTMLDivElement;
const editorEl = document.getElementById('editor') as HTMLElement;

let currentDocId: string | null = null;
let currentIr: DocumentIr | null = null;
let overlays: OverlayHandle[] = [];

function resetEditor() {
  overlays.forEach((overlay) => overlay.dispose());
  overlays = [];
  editorEl.innerHTML = '';
}

function arrayBufferToBase64(buffer: ArrayBuffer): string {
  const bytes = new Uint8Array(buffer);
  let binary = '';
  bytes.forEach((b) => (binary += String.fromCharCode(b)));
  return btoa(binary);
}

async function handleTransform(op: PatchOp) {
  if (!currentDocId) return;
  try {
    const response = await applyPatch(currentDocId, [op]);
    if (response.updatedPdf) {
      statusEl.textContent = 'Patch applied successfully.';
    }
  } catch (error) {
    console.error(error);
    statusEl.textContent = `Failed to apply patch: ${String(error)}`;
  }
}

async function loadDocument(file: File) {
  statusEl.textContent = 'Loading document…';
  const buffer = await file.arrayBuffer();
  const base64 = arrayBufferToBase64(buffer);

  try {
    const openResponse = await openDocument(base64, file.name);
    currentDocId = openResponse.docId;
    currentIr = await fetchIr(openResponse.docId);

    const pdfPages = await renderPdf(new Uint8Array(buffer), editorEl);
    editorEl.innerHTML = '';

    pdfPages.forEach((pageRender) => {
      const wrapper = document.createElement('div');
      wrapper.className = 'page-wrapper';
      wrapper.style.width = `${pageRender.widthPx}px`;
      wrapper.style.height = `${pageRender.heightPx}px`;
      wrapper.appendChild(pageRender.canvas);
      editorEl.appendChild(wrapper);

      const pageIr = currentIr?.pages.find((p) => p.index === pageRender.pageNumber - 1);
      if (pageIr) {
        const overlay = setupOverlay(wrapper, pageRender, pageIr, {
          onTransform: handleTransform
        });
        overlays.push(overlay);
      }
    });

    downloadBtn.disabled = false;
    statusEl.textContent = 'Document ready. Drag controllers to send patches.';
  } catch (error) {
    console.error(error);
    statusEl.textContent = `Failed to open document: ${String(error)}`;
    currentDocId = null;
    currentIr = null;
    resetEditor();
  }
}

fileInput.addEventListener('change', () => {
  const [file] = fileInput.files ?? [];
  if (file) {
    resetEditor();
    loadDocument(file);
  }
});

downloadBtn.addEventListener('click', async () => {
  if (!currentDocId) return;
  try {
    const blob = await downloadPdf(currentDocId);
    const url = URL.createObjectURL(blob);
    const anchor = document.createElement('a');
    anchor.href = url;
    anchor.download = 'edited.pdf';
    anchor.click();
    URL.revokeObjectURL(url);
  } catch (error) {
    console.error(error);
    statusEl.textContent = `Failed to download: ${String(error)}`;
  }
});
