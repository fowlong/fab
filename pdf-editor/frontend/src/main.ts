import './styles.css';
import { PdfPreview } from './pdfPreview';
import { FabricOverlayManager } from './fabricOverlay';
import { downloadPdf, fetchIR, openDocument, sendPatch } from './api';
import type { DocumentIR, PatchOperation, TransformPatch } from './types';

class App {
  private appEl: HTMLElement;
  private sidebar: HTMLElement;
  private main: HTMLElement;
  private previewRoot: HTMLElement;
  private overlayRoot: HTMLElement;
  private preview: PdfPreview;
  private overlay: FabricOverlayManager;
  private statusEl: HTMLElement;
  private docId: string | null = null;
  private ir: DocumentIR | null = null;

  constructor(appEl: HTMLElement) {
    this.appEl = appEl;
    this.sidebar = document.createElement('div');
    this.sidebar.className = 'sidebar';
    this.main = document.createElement('div');
    this.main.className = 'main';

    this.previewRoot = document.createElement('div');
    this.previewRoot.className = 'canvas-stack';

    this.overlayRoot = document.createElement('div');
    this.overlayRoot.className = 'overlay-root';

    const stage = document.createElement('div');
    stage.className = 'stage';
    stage.appendChild(this.previewRoot);
    stage.appendChild(this.overlayRoot);

    this.main.appendChild(stage);

    this.appEl.appendChild(this.sidebar);
    this.appEl.appendChild(this.main);

    this.preview = new PdfPreview(this.previewRoot);
    this.overlay = new FabricOverlayManager(this.overlayRoot, {
      onTransform: async (targetId, pageIndex, delta, kind) => {
        if (!this.docId) return;
        const op: TransformPatch = {
          op: 'transform',
          target: { page: pageIndex, id: targetId },
          deltaMatrixPt: delta,
          kind
        };
        await this.applyPatch([op]);
      }
    });

    this.statusEl = document.createElement('div');
    this.statusEl.className = 'status';
    this.sidebar.appendChild(this.buildUpload());
    this.sidebar.appendChild(this.buildToolbar());
    this.sidebar.appendChild(this.statusEl);
    this.setStatus('Drop a PDF to begin.');
  }

  private buildUpload(): HTMLElement {
    const wrapper = document.createElement('div');
    const label = document.createElement('label');
    label.textContent = 'Open PDF';
    label.style.fontWeight = '600';
    const input = document.createElement('input');
    input.type = 'file';
    input.accept = 'application/pdf';
    input.addEventListener('change', async () => {
      const file = input.files?.[0];
      if (!file) return;
      try {
        this.setStatus('Loading PDF…');
        const fileData = new Uint8Array(await file.arrayBuffer());
        await this.preview.render(fileData);
        const result = await openDocument(file);
        this.docId = result.docId;
        this.ir = result.ir;
        this.mountOverlay();
        this.setStatus('Ready.');
      } catch (err) {
        console.error(err);
        this.setStatus('Failed to load PDF.');
      }
    });
    wrapper.appendChild(label);
    wrapper.appendChild(input);
    return wrapper;
  }

  private buildToolbar(): HTMLElement {
    const toolbar = document.createElement('div');
    toolbar.className = 'toolbar';

    const downloadBtn = document.createElement('button');
    downloadBtn.textContent = 'Download';
    downloadBtn.addEventListener('click', async () => {
      if (!this.docId) return;
      const blob = await downloadPdf(this.docId);
      const url = URL.createObjectURL(blob);
      const link = document.createElement('a');
      link.href = url;
      link.download = 'document.pdf';
      link.click();
      URL.revokeObjectURL(url);
    });

    toolbar.appendChild(downloadBtn);
    return toolbar;
  }

  private async applyPatch(ops: PatchOperation[]): Promise<void> {
    if (!this.docId) return;
    try {
      await sendPatch(this.docId, ops);
      if (this.docId) {
        this.ir = await fetchIR(this.docId);
        this.mountOverlay();
      }
      this.setStatus('Changes saved.');
    } catch (err) {
      console.error(err);
      this.setStatus('Failed to apply change.');
    }
  }

  private mountOverlay(): void {
    if (!this.ir) return;
    const containers: HTMLElement[] = [];
    this.overlayRoot.innerHTML = '';
    this.ir.pages.forEach((page, index) => {
      const previewCanvas = this.preview.getCanvasForPage(index);
      if (!previewCanvas) return;
      const wrapper = document.createElement('div');
      wrapper.className = 'page-layer';
      wrapper.style.position = 'relative';
      wrapper.style.width = `${previewCanvas.width}px`;
      wrapper.style.height = `${previewCanvas.height}px`;
      wrapper.style.marginBottom = '1.5rem';
      const placeholder = document.createElement('div');
      placeholder.style.position = 'absolute';
      placeholder.style.inset = '0';
      wrapper.appendChild(placeholder);
      this.overlayRoot.appendChild(wrapper);
      containers[index] = placeholder;
    });
    this.overlay.mountPages(this.ir.pages, containers);
  }

  private setStatus(message: string): void {
    this.statusEl.textContent = message;
  }
}

const appRoot = document.getElementById('app');
if (!appRoot) {
  throw new Error('Missing #app container');
}

new App(appRoot);
