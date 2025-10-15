import { fabric } from 'fabric';

import type { DocumentIr, PageObject } from './types';
import { createFabricMeta, type FabricMeta } from './mapping';
import { pxToPtMatrix } from './coords';

export class FabricOverlay {
  private readonly container: HTMLElement;
  private overlays: fabric.Canvas[] = [];
  private metas = new Map<string, FabricMeta>();

  constructor(container: HTMLElement) {
    this.container = container;
  }

  reset(): void {
    for (const canvas of this.overlays) {
      const element = canvas.getElement();
      canvas.dispose();
      element?.remove();
    }
    this.overlays = [];
    this.metas.clear();
  }

  attach(ir: DocumentIr): void {
    this.reset();
    const pageLayers = Array.from(this.container.querySelectorAll<HTMLDivElement>('.page-layer'));
    pageLayers.forEach((layer, index) => {
      const pdfCanvas = layer.querySelector<HTMLCanvasElement>('canvas.pdf-preview-canvas');
      if (!pdfCanvas) {
        return;
      }
      const overlay = this.createOverlayForCanvas(layer, pdfCanvas.width, pdfCanvas.height);
      this.populateOverlay(
        overlay,
        index,
        ir.pages[index]?.objects ?? [],
        ir.pages[index]?.heightPt ?? 0,
      );
    });
  }

  private createOverlayForCanvas(layer: HTMLDivElement, width: number, height: number): fabric.Canvas {
    let overlayEl = layer.querySelector<HTMLCanvasElement>('canvas.fabric-overlay-canvas');
    if (!overlayEl) {
      overlayEl = document.createElement('canvas');
      overlayEl.className = 'fabric-overlay-canvas';
      layer.appendChild(overlayEl);
    }
    overlayEl.width = width;
    overlayEl.height = height;

    const overlay = new fabric.Canvas(overlayEl, {
      selection: false,
    });
    this.overlays.push(overlay);
    return overlay;
  }

  private populateOverlay(
    canvas: fabric.Canvas,
    pageIndex: number,
    objects: PageObject[],
    heightPt: number,
  ): void {
    for (const object of objects) {
      const rect = new fabric.Rect({
        left: 0,
        top: 0,
        width: 10,
        height: 10,
        fill: 'rgba(0,0,0,0)',
        stroke: '#00aaff',
        strokeDashArray: [4, 4],
      });
      const matrix = pxToPtMatrix(heightPt);
      const meta = createFabricMeta(pageIndex, object, matrix);
      this.metas.set(object.id, meta);
      canvas.add(rect);
    }
  }
}

