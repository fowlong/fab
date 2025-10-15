import './styles.css';
import { PdfPreview } from './pdfPreview';
import { FabricOverlayManager } from './fabricOverlay';
import type { DocumentIR } from './types';

const app = document.querySelector<HTMLDivElement>('#app');
if (!app) {
  throw new Error('App container missing');
}

app.innerHTML = `
  <div class="layout">
    <aside class="sidebar">
      <h1>PDF Editor</h1>
      <p class="sidebar__intro">Upload a PDF or load the bundled sample to inspect the IR-driven overlay.</p>
      <button id="load-sample" class="button">Load sample document</button>
      <label class="button button--secondary">
        <span>Select PDF…</span>
        <input id="file-input" type="file" accept="application/pdf" hidden />
      </label>
      <div id="status" class="status"></div>
    </aside>
    <section class="editor">
      <div id="page-stack" class="page-stack"></div>
    </section>
  </div>
`;

const statusEl = document.getElementById('status') as HTMLDivElement;
const pageStack = document.getElementById('page-stack') as HTMLDivElement;

const pdfContainer = document.createElement('div');
const preview = new PdfPreview(pdfContainer);
const overlayManager = new FabricOverlayManager();

const sampleIr: DocumentIR = {
  pages: [
    {
      index: 0,
      widthPt: 595.276,
      heightPt: 841.89,
      objects: [
        {
          id: 't:42',
          kind: 'text',
          pdfRef: { obj: 187, gen: 0 },
          btSpan: { start: 12034, end: 12345, streamObj: 155 },
          Tm: [1, 0, 0, 1, 100.2, 700.5],
          font: { resName: 'F2', size: 10.5, type: 'Type0' },
          unicode: 'Invoice #01234',
          glyphs: [
            { gid: 123, dx: 500, dy: 0 },
            { gid: 87, dx: 480, dy: 0 },
          ],
          bbox: [98.4, 688.0, 210.0, 705.0],
        },
        {
          id: 'img:9',
          kind: 'image',
          pdfRef: { obj: 200, gen: 0 },
          xObject: 'Im7',
          cm: [120, 0, 0, 90, 300.0, 500.0],
          bbox: [300.0, 500.0, 420.0, 590.0],
        },
      ],
    },
  ],
};

async function loadSample() {
  setStatus('Loading bundled sample…');
  const response = await fetch('/sample.pdf');
  const arrayBuffer = await response.arrayBuffer();
  await render(arrayBuffer, sampleIr);
  setStatus('Sample loaded.');
}

async function render(pdfData: ArrayBuffer, ir: DocumentIR) {
  pageStack.innerHTML = '';
  pdfContainer.innerHTML = '';
  await preview.load(pdfData);
  const sizes = preview.getSizes();
  const pdfCanvases = Array.from(pdfContainer.querySelectorAll('canvas'));
  const overlayWrappers: HTMLElement[] = [];

  pdfCanvases.forEach((canvas, index) => {
    const wrapper = document.createElement('div');
    wrapper.className = 'page-wrapper';
    wrapper.style.width = `${canvas.width}px`;
    wrapper.style.height = `${canvas.height}px`;
    const pdfLayer = document.createElement('div');
    pdfLayer.className = 'page-wrapper__pdf';
    pdfLayer.appendChild(canvas);
    const overlayLayer = document.createElement('div');
    overlayLayer.className = 'page-wrapper__overlay';
    wrapper.appendChild(pdfLayer);
    wrapper.appendChild(overlayLayer);
    pageStack.appendChild(wrapper);
    overlayWrappers[index] = overlayLayer;
  });

  overlayManager.populate(ir, overlayWrappers, sizes);
}

function setStatus(message: string) {
  statusEl.textContent = message;
}

const loadSampleButton = document.getElementById('load-sample') as HTMLButtonElement;
loadSampleButton.addEventListener('click', () => {
  loadSample().catch((err) => setStatus(`Failed to load sample: ${err}`));
});

const fileInput = document.getElementById('file-input') as HTMLInputElement;
fileInput.addEventListener('change', async (event) => {
  const input = event.target as HTMLInputElement;
  if (!input.files || input.files.length === 0) {
    return;
  }
  const file = input.files[0];
  setStatus(`Loaded local file: ${file.name}. Backend integration pending.`);
});

setStatus('Ready. Load the sample to see placeholder overlays.');
