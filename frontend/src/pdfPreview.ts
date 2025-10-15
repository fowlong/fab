import type { DocumentIR } from './types';

export async function initialisePdfPreview(
  container: HTMLElement,
  ir: DocumentIR
) {
  for (const page of ir.pages) {
    const wrapper = document.createElement('div');
    wrapper.className = 'page-wrapper';

    const canvas = document.createElement('canvas');
    canvas.width = Math.round((page.widthPt / 72) * 96);
    canvas.height = Math.round((page.heightPt / 72) * 96);
    canvas.dataset.pageIndex = page.index.toString();
    canvas.className = 'pdf-canvas';

    wrapper.appendChild(canvas);
    const overlay = document.createElement('canvas');
    overlay.width = canvas.width;
    overlay.height = canvas.height;
    overlay.id = `fabric-p${page.index}`;
    overlay.className = 'fabric-overlay';
    wrapper.appendChild(overlay);

    container.appendChild(wrapper);
  }

  // TODO: integrate pdfjs-dist rendering here.
}
