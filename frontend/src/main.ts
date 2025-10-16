import './styles.css';
import { PdfPreview } from './pdfPreview';
import { FabricOverlay } from './fabricOverlay';
import { download, getIR, open, patch } from './api';
import type { DocumentIR, Matrix, PageObject, PatchOperation } from './types';
import { invert, multiply } from './coords';

const app = document.querySelector<HTMLDivElement>('#app');
if (!app) {
  throw new Error('Application container is missing.');
}

type LoadedState = {
  docId: string;
  ir: DocumentIR;
};

let state: LoadedState | null = null;
let preview: PdfPreview;
let overlay: FabricOverlay | null = null;
let busy = false;

app.innerHTML = `
  <div class="layout">
    <aside class="sidebar">
      <h1>Transform stage</h1>
      <label class="button">
        <span>Select PDF…</span>
        <input id="file-input" type="file" accept="application/pdf" />
      </label>
      <button id="download-btn" class="button button--secondary" disabled>
        Download updated PDF
      </button>
      <div id="status" class="status"></div>
    </aside>
    <section class="editor">
      <div class="page-stack">
        <div id="page-wrapper" class="page-wrapper" hidden>
          <div id="pdf-layer" class="page-wrapper__pdf"></div>
          <div id="overlay-layer" class="page-wrapper__overlay"></div>
        </div>
      </div>
    </section>
  </div>
`;

const fileInput = document.getElementById('file-input') as HTMLInputElement;
const downloadBtn = document.getElementById('download-btn') as HTMLButtonElement;
const statusEl = document.getElementById('status') as HTMLDivElement;
const pdfLayer = document.getElementById('pdf-layer') as HTMLDivElement;
const overlayLayer = document.getElementById('overlay-layer') as HTMLDivElement;
const pageWrapper = document.getElementById('page-wrapper') as HTMLDivElement;

preview = new PdfPreview(pdfLayer);

function setStatus(message: string) {
  statusEl.textContent = message;
}

async function initialiseFromFile(file: File) {
  if (busy) {
    setStatus('Another action is already underway.');
    return;
  }
  busy = true;
  try {
    setStatus('Uploading PDF…');
    const arrayBuffer = await file.arrayBuffer();
    const { docId } = await open(file);

    setStatus('Building page model…');
    const ir = await getIR(docId);
    if (ir.pages.length === 0) {
      throw new Error('The document does not expose page 0.');
    }
    const page = ir.pages[0];

    const metrics = await preview.render(arrayBuffer);
    pageWrapper.style.width = `${metrics.widthPx}px`;
    pageWrapper.style.height = `${metrics.heightPx}px`;
    pageWrapper.hidden = false;

    overlay?.dispose();
    overlay = new FabricOverlay(
      overlayLayer,
      { width: metrics.widthPx, height: metrics.heightPx },
      page.heightPt,
      async ({ id, kind, pageIndex, deltaMatrixPt }) => {
        await applyTransform({ id, kind, pageIndex, deltaMatrixPt });
      },
    );
    overlay.setPage(page);

    state = { docId, ir };
    downloadBtn.disabled = false;
    setStatus('Document ready. Drag, rotate, or scale the blue guides.');
  } catch (error) {
    console.error(error);
    overlay?.dispose();
    overlay = null;
    state = null;
    downloadBtn.disabled = true;
    pageWrapper.hidden = true;
    setStatus(`Failed to load PDF: ${(error as Error).message}`);
  } finally {
    busy = false;
  }
}

async function applyTransform(args: {
  id: string;
  kind: 'text' | 'image';
  pageIndex: number;
  deltaMatrixPt: Matrix;
}) {
  if (!state) {
    throw new Error('No document is loaded.');
  }

  const { docId, ir } = state;
  setStatus('Applying transform…');
  const op: PatchOperation = {
    op: 'transform',
    target: { page: args.pageIndex, id: args.id },
    deltaMatrixPt: args.deltaMatrixPt,
    kind: args.kind,
  };

  try {
    const response = await patch(docId, [op]);
    mutateIr(ir, args);
    if (response.updatedPdf) {
      const buffer = decodeDataUrl(response.updatedPdf);
      const metrics = await preview.render(buffer);
      pageWrapper.style.width = `${metrics.widthPx}px`;
      pageWrapper.style.height = `${metrics.heightPx}px`;
    }
    setStatus('Transform applied.');
  } catch (error) {
    setStatus(`Transform failed: ${(error as Error).message}`);
    throw error;
  }
}

function mutateIr(ir: DocumentIR, args: {
  id: string;
  kind: 'text' | 'image';
  pageIndex: number;
  deltaMatrixPt: Matrix;
}) {
  const page = ir.pages[args.pageIndex];
  if (!page) {
    return;
  }
  const index = page.objects.findIndex((obj) => obj.id === args.id);
  if (index < 0) {
    return;
  }
  const object = page.objects[index];
  const baseMatrix = object.kind === 'text' ? object.Tm : object.cm;
  const updatedMatrix = multiply(args.deltaMatrixPt, baseMatrix);
  const updatedBbox = computeUpdatedBbox(object, baseMatrix, updatedMatrix);

  if (object.kind === 'text') {
    object.Tm = updatedMatrix;
  } else {
    object.cm = updatedMatrix;
  }
  if (updatedBbox) {
    object.bbox = updatedBbox;
  }
}

function computeUpdatedBbox(
  object: PageObject,
  baseMatrix: Matrix,
  nextMatrix: Matrix,
): [number, number, number, number] | null {
  const [x0, y0, x1, y1] = object.bbox;
  try {
    const inverse = invert(baseMatrix);
    const localTopLeft = transformPoint(inverse, { x: x0, y: y1 });
    const localTopRight = transformPoint(inverse, { x: x1, y: y1 });
    const localBottomLeft = transformPoint(inverse, { x: x0, y: y0 });
    const localBottomRight = transformPoint(inverse, { x: x1, y: y0 });

    const corners = [
      transformPoint(nextMatrix, localTopLeft),
      transformPoint(nextMatrix, localTopRight),
      transformPoint(nextMatrix, localBottomLeft),
      transformPoint(nextMatrix, localBottomRight),
    ];

    const xs = corners.map((pt) => pt.x);
    const ys = corners.map((pt) => pt.y);
    return [
      Math.min(...xs),
      Math.min(...ys),
      Math.max(...xs),
      Math.max(...ys),
    ];
  } catch (error) {
    console.warn('BBox update skipped:', error);
    return null;
  }
}

function transformPoint(matrix: Matrix, point: { x: number; y: number }) {
  const [a, b, c, d, e, f] = matrix;
  return {
    x: a * point.x + c * point.y + e,
    y: b * point.x + d * point.y + f,
  };
}

function decodeDataUrl(dataUrl: string): ArrayBuffer {
  const [, base64 = dataUrl] = dataUrl.split(',');
  const binary = atob(base64);
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i += 1) {
    bytes[i] = binary.charCodeAt(i);
  }
  return bytes.buffer;
}

fileInput.addEventListener('change', async () => {
  const [file] = fileInput.files ?? [];
  if (!file) {
    return;
  }
  await initialiseFromFile(file);
  fileInput.value = '';
});

downloadBtn.addEventListener('click', () => {
  if (!state) {
    setStatus('Load a PDF before downloading.');
    return;
  }
  download(state.docId);
});

setStatus('Select a PDF to begin.');
