import './styles.css';
import { initPdfPreview } from './pdfPreview';
import { initFabricOverlay } from './fabricOverlay';
import { loadInitialDocument } from './api';
import type { EditorContext } from './types';

const app = document.getElementById('app');
if (!app) {
  throw new Error('Failed to find #app container');
}

app.innerHTML = `
  <div class="layout">
    <header>
      <h1>PDF Editor MVP</h1>
      <div class="toolbar">
        <button id="open-file">Open PDF</button>
        <button id="download">Download</button>
      </div>
    </header>
    <main>
      <aside class="sidebar" id="thumbnails"></aside>
      <section class="editor" id="editor"></section>
    </main>
    <footer id="status">Ready.</footer>
  </div>
  <input type="file" id="file-input" accept="application/pdf" hidden />
`;

const fileInput = document.getElementById('file-input') as HTMLInputElement;
const openButton = document.getElementById('open-file');
const downloadButton = document.getElementById('download');
const editor = document.getElementById('editor');
const status = document.getElementById('status');

if (!fileInput || !openButton || !downloadButton || !editor || !status) {
  throw new Error('Missing layout elements');
}

const ctx: EditorContext = {
  docId: null,
  pages: [],
  overlayByPage: new Map(),
  setStatus(message) {
    status.textContent = message;
  }
};

openButton.addEventListener('click', () => fileInput.click());
fileInput.addEventListener('change', async () => {
  if (!fileInput.files || fileInput.files.length === 0) {
    return;
  }

  const file = fileInput.files[0];
  ctx.setStatus(`Opening ${file.name}...`);
  try {
    const { docId, ir } = await loadInitialDocument(file);
    ctx.docId = docId;
    ctx.pages = ir.pages;
    ctx.overlayByPage.clear();
    ctx.setStatus(`Loaded ${file.name}`);

    editor.innerHTML = '';
    const preview = await initPdfPreview(docId, ir, editor);
    initFabricOverlay(ctx, preview);
  } catch (err) {
    console.error(err);
    ctx.setStatus('Failed to load PDF');
  }
});

downloadButton.addEventListener('click', async () => {
  if (!ctx.docId) {
    ctx.setStatus('Load a PDF first');
    return;
  }
  ctx.setStatus('Downloading PDF...');
  try {
    const url = `${(window as any).__API_BASE__ ?? 'http://localhost:8787'}/api/pdf/${ctx.docId}`;
    const response = await fetch(url);
    const blob = await response.blob();
    const objectUrl = URL.createObjectURL(blob);
    const anchor = document.createElement('a');
    anchor.href = objectUrl;
    anchor.download = 'edited.pdf';
    anchor.click();
    URL.revokeObjectURL(objectUrl);
    ctx.setStatus('Downloaded PDF');
  } catch (err) {
    console.error(err);
    ctx.setStatus('Failed to download');
  }
});
