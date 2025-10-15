import './styles.css';
import { fabric } from 'fabric';
import { downloadPdf, openDocument, postPatch } from './api';
import type { DocumentIR, PatchOp, ToastMessage } from './types';
import { createOverlayCanvas, installInteractionHandlers, populateOverlay } from './fabricOverlay';
import type { Matrix } from './coords';
import { loadPdf, renderPage } from './pdfPreview';

const app = document.querySelector<HTMLDivElement>('#app');
if (!app) {
  throw new Error('#app mount element missing');
}

app.innerHTML = `
  <header>
    <div class="toolbar">
      <label class="file-upload">
        <input type="file" id="file-input" accept="application/pdf" hidden />
        <span>Open PDF</span>
      </label>
      <button id="download-btn" disabled>Download PDF</button>
    </div>
    <div class="toolbar" id="status-bar"></div>
  </header>
  <main>
    <aside id="thumbnails"></aside>
    <section class="editor" id="editor"></section>
  </main>
  <div class="toast-container" id="toast-container"></div>
`;

const fileInput = document.getElementById('file-input') as HTMLInputElement;
const downloadBtn = document.getElementById('download-btn') as HTMLButtonElement;
const editor = document.getElementById('editor') as HTMLElement;
const thumbnails = document.getElementById('thumbnails') as HTMLElement;
const toastContainer = document.getElementById('toast-container') as HTMLElement;

let docId: string | null = null;
let currentIr: DocumentIR | null = null;
let currentPdf: ArrayBuffer | null = null;
const overlayByPage = new Map<number, fabric.Canvas>();

fileInput.addEventListener('change', async () => {
  if (!fileInput.files || fileInput.files.length === 0) return;
  const file = fileInput.files[0];
  try {
    const { docId: newDocId, ir } = await openDocument(file);
    docId = newDocId;
    currentIr = ir;
    downloadBtn.disabled = false;
    const buffer = await file.arrayBuffer();
    currentPdf = buffer;
    await renderDocument(ir, buffer);
    pushToast({ id: crypto.randomUUID(), text: 'PDF loaded', tone: 'success' });
  } catch (err) {
    pushToast({ id: crypto.randomUUID(), text: (err as Error).message, tone: 'error' });
  }
});

downloadBtn.addEventListener('click', async () => {
  if (!docId) return;
  try {
    const blob = await downloadPdf(docId);
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = 'edited.pdf';
    a.click();
    URL.revokeObjectURL(url);
  } catch (err) {
    pushToast({ id: crypto.randomUUID(), text: (err as Error).message, tone: 'error' });
  }
});

async function renderDocument(ir: DocumentIR, pdfData: ArrayBuffer) {
  editor.innerHTML = '';
  thumbnails.innerHTML = '';
  overlayByPage.clear();

  const pdf = await loadPdf(pdfData);

  for (const page of ir.pages) {
    const pageWrapper = document.createElement('div');
    pageWrapper.className = 'pdf-page';
    pageWrapper.style.width = `${page.widthPt * (96 / 72)}px`;
    pageWrapper.style.height = `${page.heightPt * (96 / 72)}px`;

    const previewCanvas = document.createElement('canvas');
    previewCanvas.className = 'pdf-preview';
    pageWrapper.appendChild(previewCanvas);
    await renderPage(pdf, page, previewCanvas);

    const overlayCanvas = createOverlayCanvas(pageWrapper, previewCanvas.width, previewCanvas.height);
    overlayByPage.set(page.index, overlayCanvas);
    populateOverlay(overlayCanvas, page);

    const thumb = pageWrapper.cloneNode(true) as HTMLElement;
    thumb.classList.add('page-thumb');
    thumbnails.appendChild(thumb);
    editor.appendChild(pageWrapper);
  }

  if (docId && currentIr) {
    installInteractionHandlers({
      docId,
      ir: currentIr,
      pdfData,
      canvasByPage: overlayByPage,
      dispatchToast: (message, tone = 'info') => pushToast({ id: crypto.randomUUID(), text: message, tone }),
      onPdfUpdated: (response) => {
        if (response.updatedPdf) {
          currentPdf = decodeBase64(response.updatedPdf);
        }
      },
    });
  }
}

function pushToast(toast: ToastMessage) {
  const el = document.createElement('div');
  el.className = 'toast';
  el.dataset.tone = toast.tone;
  el.textContent = toast.text;
  toastContainer.appendChild(el);
  setTimeout(() => el.remove(), 4000);
}

function decodeBase64(dataUrl: string): ArrayBuffer {
  const [, base64] = dataUrl.split(',', 2);
  const bytes = atob(base64 ?? '');
  const buffer = new Uint8Array(bytes.length);
  for (let i = 0; i < bytes.length; i += 1) {
    buffer[i] = bytes.charCodeAt(i);
  }
  return buffer.buffer;
}

export function applyPatch(patch: PatchOp) {
  if (!docId || !currentIr) return;
  postPatch(docId, [patch])
    .then((response) => {
      if (response.updatedPdf) {
        currentPdf = decodeBase64(response.updatedPdf);
      }
      pushToast({ id: crypto.randomUUID(), text: 'Patch applied', tone: 'success' });
    })
    .catch((err) => pushToast({ id: crypto.randomUUID(), text: (err as Error).message, tone: 'error' }));
}

export function getOverlay(docPage: number) {
  return overlayByPage.get(docPage) ?? null;
}

export function getPdfBuffer() {
  return currentPdf;
}

export function setBaseMatrix(pageIndex: number, objectId: string, matrix: Matrix) {
  const canvas = overlayByPage.get(pageIndex);
  if (!canvas) return;
  const object = canvas.getObjects().find((o) => (o as any).__meta?.id === objectId);
  if (object) {
    (object as any).__meta.baseMatrixPx = matrix;
  }
}
