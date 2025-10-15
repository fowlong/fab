import './styles.css';
import { createPdfPreview } from './pdfPreview';
import { createOverlayController } from './fabricOverlay';
import { loadDocumentIR } from './api';

const app = document.querySelector<HTMLDivElement>('#app');
if (!app) {
  throw new Error('App root not found');
}

const state = {
  docId: '',
};

async function init() {
  const uploadInput = document.createElement('input');
  uploadInput.type = 'file';
  uploadInput.accept = 'application/pdf';

  const previewContainer = document.createElement('div');
  previewContainer.className = 'preview-container';

  const toolbar = document.createElement('div');
  toolbar.className = 'toolbar';

  const downloadButton = document.createElement('button');
  downloadButton.textContent = 'Download PDF';
  downloadButton.disabled = true;

  toolbar.append(uploadInput, downloadButton);
  app.append(toolbar, previewContainer);

  uploadInput.addEventListener('change', async () => {
    const file = uploadInput.files?.[0];
    if (!file) return;

    const { docId, ir } = await loadDocumentIR(file);
    state.docId = docId;
    downloadButton.disabled = false;

    previewContainer.innerHTML = '';
    const previews = await createPdfPreview(ir.pages, previewContainer);
    createOverlayController({ docId, ir, previews });
  });
}

init().catch((err) => console.error(err));
