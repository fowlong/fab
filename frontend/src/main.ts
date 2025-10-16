import './styles.css';
import { open, getIR, patch, download } from './api';
import { PdfPreview } from './pdfPreview';
import { FabricOverlay } from './fabricOverlay';
import type { DocumentIR, PatchOperation } from './types';

const app = document.querySelector<HTMLDivElement>('#app');
if (!app) {
  throw new Error('App container missing');
}

app.innerHTML = `
  <div class="layout">
    <aside class="sidebar">
      <h1>PDF Transform MVP</h1>
      <p class="sidebar__intro">Upload a PDF with editable text and images. Drag the overlay handles to send true PDF transforms to the backend.</p>
      <label class="button button--secondary">
        <span>Select PDF…</span>
        <input id="file-input" type="file" accept="application/pdf" />
      </label>
      <button id="download" class="button" disabled>Download updated PDF</button>
      <div id="status" class="status"></div>
    </aside>
    <section class="editor">
      <div id="page-stack" class="page-stack"></div>
    </section>
  </div>
`;

const statusEl = document.getElementById('status') as HTMLDivElement;
const pageStack = document.getElementById('page-stack') as HTMLDivElement;
const fileInput = document.getElementById('file-input') as HTMLInputElement;
const downloadButton = document.getElementById('download') as HTMLButtonElement;

const preview = new PdfPreview();
let overlay: FabricOverlay | null = null;
let currentDocId: string | null = null;

fileInput.addEventListener('change', async (event) => {
  const input = event.target as HTMLInputElement;
  if (!input.files || input.files.length === 0) {
    return;
  }
  const file = input.files[0];
  try {
    await handleOpen(file);
  } catch (error) {
    setStatus(`Failed to open PDF: ${error}`);
  }
});

downloadButton.addEventListener('click', async () => {
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
    document.body.appendChild(anchor);
    anchor.click();
    anchor.remove();
    URL.revokeObjectURL(url);
    setStatus('Download complete.');
  } catch (error) {
    setStatus(`Download failed: ${error}`);
  }
});

async function handleOpen(file: File) {
  setStatus('Uploading PDF…');
  const response = await open(file);
  currentDocId = response.docId;
  const ir = await getIR(currentDocId);
  const data = await file.arrayBuffer();
  await renderPage(data, ir);
  downloadButton.disabled = false;
  setStatus('Ready to edit. Drag a controller to transform the PDF.');
}

async function renderPage(pdfData: ArrayBuffer, ir: DocumentIR) {
  const page = ir.pages[0];
  if (!page) {
    throw new Error('IR missing page 0');
  }

  pageStack.innerHTML = '';
  overlay?.dispose();

  const wrapper = document.createElement('div');
  wrapper.className = 'page-wrapper';
  const pdfLayer = document.createElement('div');
  pdfLayer.className = 'page-wrapper__pdf';
  const overlayLayer = document.createElement('div');
  overlayLayer.className = 'page-wrapper__overlay';
  wrapper.appendChild(pdfLayer);
  wrapper.appendChild(overlayLayer);
  pageStack.appendChild(wrapper);

  await preview.render(pdfLayer, pdfData);
  const pixelSize = preview.getPixelSize();
  const pageSizePt = preview.getPageSizePt();
  if (!pixelSize || !pageSizePt) {
    throw new Error('Failed to measure page');
  }

  overlay = new FabricOverlay(overlayLayer, pixelSize, pageSizePt.height, {
    onTransform: handleTransform,
  });
  overlay.render(page);
}

const handleTransform = async (op: PatchOperation): Promise<boolean> => {
  if (!currentDocId) {
    return false;
  }
  try {
    setStatus('Applying transform…');
    const response = await patch(currentDocId, [op]);
    if (response.updatedPdf) {
      const buffer = await dataUriToArrayBuffer(response.updatedPdf);
      const ir = await getIR(currentDocId);
      await renderPage(buffer, ir);
      setStatus('Transform applied.');
    } else {
      setStatus('No changes detected.');
    }
    return true;
  } catch (error) {
    setStatus(`Transform failed: ${error}`);
    return false;
  }
};

function setStatus(message: string) {
  statusEl.textContent = message;
}

async function dataUriToArrayBuffer(uri: string): Promise<ArrayBuffer> {
  const [, base64] = uri.split(',', 2);
  if (!base64) {
    throw new Error('invalid data URI');
  }
  const binary = atob(base64);
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i += 1) {
    bytes[i] = binary.charCodeAt(i);
  }
  return bytes.buffer;
}

setStatus('Select a PDF to begin.');
