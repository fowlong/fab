import './styles.css';
import { createPdfPreview } from './pdfPreview';
import { createFabricOverlayManager } from './fabricOverlay';
import { ApiClient } from './api';
import type { DocumentIR } from './types';

const apiBase = import.meta.env.VITE_API_BASE ?? 'http://localhost:8787';
const api = new ApiClient(apiBase);

const root = document.getElementById('app');
if (!root) {
  throw new Error('Missing root element');
}

root.innerHTML = `
  <main class="layout">
    <aside class="sidebar" id="thumbnail-pane"></aside>
    <section class="editor">
      <header class="toolbar">
        <button id="tool-move" class="toolbar__button toolbar__button--active">Move/Scale</button>
        <button id="tool-edit" class="toolbar__button">Edit Text</button>
        <button id="tool-style" class="toolbar__button">Colour</button>
        <button id="tool-opacity" class="toolbar__button">Opacity</button>
        <button id="tool-download" class="toolbar__button toolbar__button--cta">Download PDF</button>
      </header>
      <section id="page-container" class="page-container"></section>
    </section>
  </main>
`;

async function bootstrap() {
  const fileInput = document.createElement('input');
  fileInput.type = 'file';
  fileInput.accept = 'application/pdf';
  fileInput.className = 'hidden';
  document.body.appendChild(fileInput);

  const docId = await new Promise<string>((resolve, reject) => {
    fileInput.addEventListener('change', async () => {
      if (!fileInput.files?.length) {
        reject(new Error('No file selected'));
        return;
      }
      try {
        const result = await api.openDocument(fileInput.files[0]);
        resolve(result.docId);
      } catch (error) {
        reject(error);
      }
    });
  });

  const ir: DocumentIR = await api.fetchIR(docId);
  const preview = await createPdfPreview({
    container: document.getElementById('page-container')!,
    ir,
  });

  createFabricOverlayManager({
    preview,
    api,
    docId,
  });
}

bootstrap().catch((error) => {
  console.error('Failed to initialise editor', error);
});
