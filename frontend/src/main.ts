import './styles.css';
import { open, getIR, patch, download, type OpenResponse } from './api';
import { PdfPreview } from './pdfPreview';
import { FabricOverlayManager, type TransformRequest } from './fabricOverlay';
import type { DocumentIR, PatchOperation } from './types';

const app = document.querySelector<HTMLDivElement>('#app');
if (!app) {
  throw new Error('App container missing');
}

type PageLayer = {
  wrapper: HTMLDivElement;
  overlay: HTMLDivElement;
};

app.innerHTML = `
  <div class="layout">
    <aside class="sidebar">
      <h1>PDF Transform MVP</h1>
      <p class="sidebar__intro">Upload a PDF to enable draggable controllers for text runs and images.</p>
      <label class="button">
        <span>Select PDF…</span>
        <input id="file-input" type="file" accept="application/pdf" hidden />
      </label>
      <button id="download-btn" class="button button--secondary" disabled>Download updated PDF</button>
      <div id="status" class="status">Idle.</div>
    </aside>
    <section class="editor">
      <div id="page-stack" class="page-stack"></div>
    </section>
  </div>
`;

const statusEl = document.getElementById('status') as HTMLDivElement;
const pageStack = document.getElementById('page-stack') as HTMLDivElement;
const fileInput = document.getElementById('file-input') as HTMLInputElement;
const downloadBtn = document.getElementById('download-btn') as HTMLButtonElement;

const previewContainer = document.createElement('div');
previewContainer.className = 'pdf-container';
pageStack.appendChild(previewContainer);

const preview = new PdfPreview(previewContainer);

let currentDocId: string | null = null;
let currentIr: DocumentIR | null = null;
let currentLayers: PageLayer[] = [];

const overlayManager = new FabricOverlayManager(async (request) => handleTransform(request));

fileInput.addEventListener('change', async (event) => {
  const input = event.target as HTMLInputElement;
  if (!input.files || input.files.length === 0) {
    return;
  }
  const file = input.files[0];
  try {
    setStatus(`Uploading ${file.name}…`);
    const response = await open(file);
    currentDocId = response.docId;
    downloadBtn.disabled = false;
    await loadDocument(response);
    setStatus('Document loaded. Drag controllers to patch the PDF.');
  } catch (err) {
    console.error(err);
    setStatus(`Open failed: ${err}`);
  }
});

downloadBtn.addEventListener('click', async () => {
  if (!currentDocId) {
    return;
  }
  try {
    setStatus('Downloading updated PDF…');
    const blob = await download(currentDocId);
    const url = URL.createObjectURL(blob);
    const anchor = document.createElement('a');
    anchor.href = url;
    anchor.download = `${currentDocId}.pdf`;
    anchor.click();
    URL.revokeObjectURL(url);
    setStatus('Download started.');
  } catch (err) {
    console.error(err);
    setStatus(`Download failed: ${err}`);
  }
});

async function loadDocument(response: OpenResponse): Promise<void> {
  if (!response.docId) {
    throw new Error('docId missing from open response');
  }
  const blob = await download(response.docId);
  const arrayBuffer = await blob.arrayBuffer();
  const renderInfo = await preview.render(arrayBuffer);
  currentIr = await getIR(response.docId);
  rebuildPageLayers(renderInfo);
  overlayManager.populate(currentIr, currentLayers.map((layer) => layer.overlay), [
    { width: renderInfo.widthPx, height: renderInfo.heightPx },
  ]);
}

async function handleTransform(request: TransformRequest): Promise<boolean> {
  if (!currentDocId) {
    return false;
  }
  const ops: PatchOperation[] = [
    {
      op: 'transform',
      target: { page: request.pageIndex, id: request.id },
      deltaMatrixPt: request.deltaMatrix,
      kind: request.kind,
    },
  ];
  try {
    setStatus('Applying transform…');
    const response = await patch(currentDocId, ops);
    if (!response.ok) {
      setStatus('Patch failed.');
      return false;
    }
    let arrayBuffer: ArrayBuffer | null = null;
    if (response.updatedPdf) {
      arrayBuffer = dataUrlToArrayBuffer(response.updatedPdf);
    }
    if (!arrayBuffer) {
      const blob = await download(currentDocId);
      arrayBuffer = await blob.arrayBuffer();
    }
    const renderInfo = await preview.render(arrayBuffer);
    currentIr = await getIR(currentDocId);
    rebuildPageLayers(renderInfo);
    overlayManager.populate(currentIr, currentLayers.map((layer) => layer.overlay), [
      { width: renderInfo.widthPx, height: renderInfo.heightPx },
    ]);
    setStatus('Transform applied.');
    return true;
  } catch (err) {
    console.error(err);
    setStatus(`Patch failed: ${err}`);
    return false;
  }
}

function rebuildPageLayers(renderInfo: Awaited<ReturnType<PdfPreview['render']>>): void {
  currentLayers.forEach((layer) => {
    layer.wrapper.remove();
  });
  currentLayers = [];
  const wrapper = document.createElement('div');
  wrapper.className = 'page-wrapper';
  wrapper.style.width = `${renderInfo.widthPx}px`;
  wrapper.style.height = `${renderInfo.heightPx}px`;

  const pdfLayer = document.createElement('div');
  pdfLayer.className = 'page-wrapper__pdf';
  pdfLayer.style.width = `${renderInfo.widthPx}px`;
  pdfLayer.style.height = `${renderInfo.heightPx}px`;

  const overlay = document.createElement('div');
  overlay.className = 'page-wrapper__overlay';
  overlay.style.width = `${renderInfo.widthPx}px`;
  overlay.style.height = `${renderInfo.heightPx}px`;

  pdfLayer.appendChild(previewContainer);
  wrapper.appendChild(pdfLayer);
  wrapper.appendChild(overlay);
  pageStack.appendChild(wrapper);
  currentLayers.push({ wrapper, overlay });
}

function dataUrlToArrayBuffer(dataUrl: string): ArrayBuffer {
  const [, base64] = dataUrl.split(',');
  const binary = window.atob(base64 ?? dataUrl);
  const length = binary.length;
  const bytes = new Uint8Array(length);
  for (let i = 0; i < length; i += 1) {
    bytes[i] = binary.charCodeAt(i);
  }
  return bytes.buffer;
}

function setStatus(message: string): void {
  statusEl.textContent = message;
}

setStatus('Select a PDF to begin.');
