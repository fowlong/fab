import { fabric } from 'fabric';
import type { DocumentIR, PageIR, PageObject, PdfMatrix } from './types';
import { controllerFromObject, getMeta } from './mapping';
import { fabricDeltaToPdfDelta } from './coords';

export interface OverlayCallbacks {
  onTransform?: (
    target: { page: number; id: string; kind: PageObject['kind'] },
    deltaMatrixPt: PdfMatrix
  ) => void | Promise<void>;
  onEditText?: (
    target: { page: number; id: string },
    initialText: string
  ) => void | Promise<void>;
}

export class FabricOverlayManager {
  private canvases = new Map<number, fabric.Canvas>();
  private pages = new Map<number, PageIR>();
  private readonly callbacks: OverlayCallbacks;

  constructor(callbacks: OverlayCallbacks = {}) {
    this.callbacks = callbacks;
  }

  mount(pages: PageIR[], containers: HTMLDivElement[]) {
    this.dispose();
    this.pages.clear();
    pages.forEach((page, index) => {
      this.pages.set(page.index, page);
      const container = containers[index];
      const overlayCanvas = document.createElement('canvas');
      overlayCanvas.id = `fabric-p${index}`;
      overlayCanvas.width = container.clientWidth;
      overlayCanvas.height = container.clientHeight;
      overlayCanvas.style.position = 'absolute';
      overlayCanvas.style.left = '0';
      overlayCanvas.style.top = '0';
      overlayCanvas.style.zIndex = '10';
      overlayCanvas.style.pointerEvents = 'auto';
      container.append(overlayCanvas);

      const fabricCanvas = new fabric.Canvas(overlayCanvas, {
        selection: true,
        uniScaleTransform: true
      });
      fabricCanvas.setDimensions({
        width: container.clientWidth,
        height: container.clientHeight
      });
      fabricCanvas.on('object:modified', (event) => this.onObjectModified(event));
      fabricCanvas.on('mouse:dblclick', (event) => this.onDoubleClick(event));
      this.canvases.set(page.index, fabricCanvas);
    });
  }

  dispose() {
    this.canvases.forEach((canvas) => canvas.dispose());
    this.canvases.clear();
    this.pages.clear();
  }

  syncPage(page: PageIR) {
    const canvas = this.canvases.get(page.index);
    if (!canvas) return;
    this.pages.set(page.index, page);
    canvas.clear();
    page.objects.forEach((object) => {
      const controller = controllerFromObject(page.index, object, page.heightPt);
      controller.set({ selectable: true, evented: true });
      controller.set('hasControls', object.kind !== 'text');
      controller.set('hoverCursor', 'move');
      controller.set('strokeDashArray', [4, 4]);
      controller.set('fill', 'rgba(0,0,0,0)');
      controller.set('strokeWidth', 1.5);
      controller.set('cornerColor', '#2ca02c');
      controller.set('transparentCorners', false);
      controller.set('objectCaching', false);
      controller.set('perPixelTargetFind', false);
      canvas.add(controller);
    });
    canvas.renderAll();
  }

  syncDocument(ir: DocumentIR) {
    ir.pages.forEach((page) => this.syncPage(page));
  }

  private onObjectModified(event: fabric.IEvent<'object:modified'>) {
    const target = event.target;
    if (!target) return;
    const meta = getMeta(target);
    if (!meta) return;
    const pageInfo = this.pages.get(meta.pageIndex);
    if (!pageInfo) return;
    const pageHeight = pageInfo.heightPt;
    const fold = meta.baseMatrix as PdfMatrix;
    const fnew = target.calcTransformMatrix() as unknown as PdfMatrix;
    try {
      const delta = fabricDeltaToPdfDelta(fold, fnew, pageHeight);
      const { onTransform } = this.callbacks;
      if (onTransform) {
        onTransform({ page: meta.pageIndex, id: meta.id, kind: meta.kind }, delta);
      }
    } catch (error) {
      console.error('Failed to compute transform delta', error);
    } finally {
      meta.baseMatrix = fnew.slice() as PdfMatrix;
      target.transformMatrix = fnew.slice() as unknown as number[];
      target.setCoords();
      target.set('dirty', true);
      target.canvas?.renderAll();
    }
  }

  private onDoubleClick(event: fabric.IEvent<'mouse:dblclick'>) {
    const target = event.target;
    if (!target) return;
    const meta = getMeta(target);
    if (!meta || meta.kind !== 'text') return;
    const { onEditText } = this.callbacks;
    if (!onEditText) return;
    onEditText({ page: meta.pageIndex, id: meta.id }, meta.id);
  }
}
