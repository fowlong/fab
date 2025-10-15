import type { DocumentIR } from './types';

export interface PdfPreviewHandle {
  container: HTMLDivElement;
  setDocument(ir: DocumentIR | null): void;
}

export async function createPdfPreview(container: HTMLDivElement): Promise<PdfPreviewHandle> {
  return {
    container,
    setDocument(ir) {
      container.innerHTML = '';
      if (!ir) {
        return;
      }
      ir.pages.forEach((page) => {
        const placeholder = document.createElement('div');
        placeholder.className = 'pdf-preview__page';
        placeholder.textContent = `Page ${page.index + 1} — ${page.widthPt.toFixed(0)}×${page.heightPt.toFixed(0)}pt`;
        container.appendChild(placeholder);
      });
    },
  };
}
