import './styles.css';

import { ApiClient } from './api';
import { PdfPreview } from './pdfPreview';
import { FabricOverlay } from './fabricOverlay';
import type { DocumentId, DocumentIr, PatchOp } from './types';

const api = new ApiClient();
let currentDoc: DocumentId | null = null;
let currentIr: DocumentIr | null = null;
let currentPdfData: ArrayBuffer = new ArrayBuffer(0);

const app = document.getElementById('app');
if (!app) {
  throw new Error('App container missing');
}

const sidebar = document.createElement('div');
sidebar.className = 'sidebar';
sidebar.innerHTML = `
  <h1>PDF Editor</h1>
  <p>Upload a PDF to parse the intermediate representation and experiment with the editing overlay.</p>
  <label class="button">
    <input id="file-input" type="file" accept="application/pdf" hidden />
    <span>Choose PDF…</span>
  </label>
  <div class="toolbar">
    <button id="download-btn" disabled>Download PDF</button>
  </div>
  <div class="status-area" id="status-area"></div>
`;

const editor = document.createElement('div');
editor.className = 'editor';
const previewContainer = document.createElement('div');
editor.appendChild(previewContainer);

app.append(sidebar, editor);

const statusArea = sidebar.querySelector<HTMLDivElement>('#status-area');
const fileInput = sidebar.querySelector<HTMLInputElement>('#file-input');
const downloadBtn = sidebar.querySelector<HTMLButtonElement>('#download-btn');

const pdfPreview = new PdfPreview(previewContainer);
const fabricOverlay = new FabricOverlay(previewContainer);

fileInput?.addEventListener('change', async (event) => {
  const target = event.target as HTMLInputElement;
  const file = target.files?.[0];
  if (!file) {
    return;
  }
  setStatus('Uploading PDF…');
  try {
    const docId = await api.open(file);
    currentDoc = docId;
    currentPdfData = await file.arrayBuffer();
    const ir = await api.fetchIr(docId);
    currentIr = ir;
    await pdfPreview.render(currentPdfData, ir);
    fabricOverlay.attach(ir);
    setStatus('PDF loaded. Use the toolbar to interact with the overlay.');
    if (downloadBtn) {
      downloadBtn.disabled = false;
    }
  } catch (error) {
    console.error(error);
    setStatus(`Failed to load PDF: ${(error as Error).message}`);
  } finally {
    target.value = '';
  }
});

downloadBtn?.addEventListener('click', async () => {
  if (!currentDoc) {
    return;
  }
  setStatus('Downloading latest PDF…');
  try {
    const blob = await api.download(currentDoc);
    const url = URL.createObjectURL(blob);
    const anchor = document.createElement('a');
    anchor.href = url;
    anchor.download = 'edited.pdf';
    anchor.click();
    URL.revokeObjectURL(url);
    setStatus('Download started.');
  } catch (error) {
    console.error(error);
    setStatus(`Download failed: ${(error as Error).message}`);
  }
});

function setStatus(message: string): void {
  if (statusArea) {
    statusArea.textContent = message;
  }
}

async function applyPatch(ops: PatchOp[]): Promise<void> {
  if (!currentDoc) {
    throw new Error('No active document');
  }
  await api.applyPatch(currentDoc, ops);
}

