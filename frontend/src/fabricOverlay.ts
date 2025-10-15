// src/fabricOverlay.ts

// Robust import that works across Fabric v4/v5 and different bundlers
import * as FabricNS from 'fabric';
const fabric: any =
  (FabricNS as any).fabric ?? // UMD-style { fabric }
  (FabricNS as any).default ?? // default export carrying the namespace
  (FabricNS as any); // namespace exports (Canvas, Rect, etc. on the root)

import type { DocumentIR, PageIR, PageObject } from './types';
import { createFabricPlaceholder } from './mapping';

type OverlayEntry = {
  // Use `any` to avoid TS complaining about types across the various export shapes
  canvas: any;
  element: HTMLCanvasElement;
};

export class FabricOverlayManager {
  private overlays = new Map<number, OverlayEntry>();

  reset() {
    for (const entry of this.overlays.values()) {
      entry.canvas.dispose();
      entry.element.remove();
    }
    this.overlays.clear();
  }

  mountOverlay(
    page: PageIR,
    wrapper: HTMLElement,
    size: { width: number; height: number },
  ) {
    const existing = this.overlays.get(page.index);
    if (existing) {
      existing.canvas.dispose();
      existing.element.remove();
    }

    const canvasEl = document.createElement('canvas');
    canvasEl.width = size.width;
    canvasEl.height = size.height;
    canvasEl.style.width = `${size.width}px`;
    canvasEl.style.height = `${size.height}px`;
    canvasEl.className = 'fabric-page-overlay';
    wrapper.appendChild(canvasEl);

    const canvas = new fabric.Canvas(canvasEl, {
      selection: true,
    });

    this.overlays.set(page.index, { canvas, element: canvasEl });
    return canvas;
  }

  populate(
    ir: DocumentIR,
    wrappers: HTMLElement[],
    pageSizes: Array<{ width: number; height: number }>,
  ) {
    this.reset();

    ir.pages.forEach((page) => {
      const wrapper = wrappers[page.index];
      const size = pageSizes[page.index];
      if (!wrapper || !size) return;

      const canvas = this.mountOverlay(page, wrapper, size);
      page.objects.forEach((obj) => this.addPlaceholder(canvas, page, obj));
      canvas.renderAll();
    });
  }

  private addPlaceholder(canvas: any, page: PageIR, obj: PageObject) {
    const placeholder = createFabricPlaceholder(canvas, page, obj);
    canvas.add(placeholder);
  }
}
