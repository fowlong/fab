import './styles.css';
import { open, getIR, patch, download } from './api';
import { PdfPreview } from './pdfPreview';
import { FabricOverlay } from './fabricOverlay';
import type { DocumentIR } from './types';

const app = document.querySelector<HTMLDivElement>('#app');
if (!app) {
  throw new Error('App container missing');
}

app.innerHTML = `
  <div class="app-shell">
    <header class="toolbar">
      <label class="control-button">
        <span>Select PDF</span>
        <input id="file-input" type="file" accept="application/pdf" />
      </label>
      <button id="download-button" class="control-button" disabled>Download updated PDF</button>
    </header>
    <p id="status" class="status">Select a PDF to begin.</p>
    <section class="viewer" id="viewer">
      <canvas id="pdf-underlay" class="pdf-underlay"></canvas>
      <div id="overlay-layer" class="overlay-layer"></div>
    </section>
  </div>
`;

const fileInput = document.getElementById('file-input') as HTMLInputElement;
const downloadButton = document.getElementById('download-button') as HTMLButtonElement;
const statusEl = document.getElementById('status') as HTMLParagraphElement;
const pdfCanvas = document.getElementById('pdf-underlay') as HTMLCanvasElement;
const overlayLayer = document.getElementById('overlay-layer') as HTMLDivElement;

const preview = new PdfPreview(pdfCanvas);
let overlay: FabricOverlay | null = null;

let currentDocId: string | null = null;
let patchInFlight = false;

fileInput.addEventListener('change', async (event) => {
  const input = event.target as HTMLInputElement;
  const file = input.files?.[0];
  if (!file) {
    return;
  }
  try {
    setStatus('Uploading PDF…');
    const [arrayBuffer, openResponse] = await Promise.all([
      file.arrayBuffer(),
      open(file),
    ]);
    currentDocId = openResponse.docId;
    await refreshFromServer(arrayBuffer);
    downloadButton.disabled = false;
    setStatus('Document loaded. Drag, rotate, or scale the controller to transform.');
  } catch (error) {
    console.error(error);
    setStatus(`Failed to load PDF: ${describeError(error)}`);
  } finally {
    fileInput.value = '';
  }
});

downloadButton.addEventListener('click', async () => {
  if (!currentDocId) {
    return;
  }
  try {
    setStatus('Preparing download…');
    const blob = await download(currentDocId);
    const url = URL.createObjectURL(blob);
    const anchor = document.createElement('a');
    anchor.href = url;
    anchor.download = `${currentDocId}.pdf`;
    anchor.click();
    URL.revokeObjectURL(url);
    setStatus('Download started.');
  } catch (error) {
    console.error(error);
    setStatus(`Download failed: ${describeError(error)}`);
  }
});

async function refreshFromServer(bufferOverride?: ArrayBuffer) {
  if (!currentDocId) {
    return;
  }
  const ir = await getIR(currentDocId);
  const buffer = bufferOverride ?? (await fetchPdfBuffer(currentDocId));
  await renderPreview(buffer, ir);
}

async function renderPreview(buffer: ArrayBuffer, ir: DocumentIR) {
  const { width, height } = await preview.render(buffer);
  overlayLayer.style.width = `${width}px`;
  overlayLayer.style.height = `${height}px`;
  overlay = new FabricOverlay(
    overlayLayer,
    async (id, kind, delta) => {
      if (!currentDocId) {
        return;
      }
      if (patchInFlight) {
        throw new Error('Another transform is still being applied.');
      }
      patchInFlight = true;
      try {
        setStatus('Applying transform…');
        const operations = [
          {
            op: 'transform' as const,
            target: { page: 0, id },
            deltaMatrixPt: delta,
            kind,
          },
        ];
        const response = await patch(currentDocId, operations);
        const updatedBuffer = response.updatedPdf
          ? decodeDataUrl(response.updatedPdf)
          : await fetchPdfBuffer(currentDocId);
        await refreshFromServer(updatedBuffer);
        setStatus('Transform applied.');
      } catch (error) {
        setStatus(`Transform failed: ${describeError(error)}`);
        throw error;
      } finally {
        patchInFlight = false;
      }
    },
    (error) => {
      setStatus(`Transform failed: ${describeError(error)}`);
    },
  );
  const pageZero = ir.pages.find((page) => page.index === 0);
  if (!pageZero) {
    throw new Error('Page 0 missing from IR.');
  }
  overlay.render(pageZero);
}

async function fetchPdfBuffer(docId: string): Promise<ArrayBuffer> {
  const blob = await download(docId);
  return blob.arrayBuffer();
}

function decodeDataUrl(dataUrl: string): ArrayBuffer {
  const [, base64] = dataUrl.split(',', 2);
  const source = base64 ?? dataUrl;
  const binary = atob(source);
  const bytes = new Uint8Array(binary.length);
  for (let index = 0; index < binary.length; index += 1) {
    bytes[index] = binary.charCodeAt(index);
  }
  return bytes.buffer;
}

function setStatus(message: string) {
  statusEl.textContent = message;
}

function describeError(error: unknown): string {
  if (error instanceof Error) {
    return error.message;
  }
  return String(error);
}
